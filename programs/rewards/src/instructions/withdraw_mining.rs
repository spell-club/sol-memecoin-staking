use crate::find_reward_pool_spl_token_account;
use crate::state::{Mining, RewardPool};
use everlend_utils::{assert_account_key, find_program_address, AccountLoader, EverlendError};
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_program::sysvar::{clock, Sysvar};

/// Instruction context
pub struct WithdrawMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_pool_spl: &'a AccountInfo<'b>,
    reward_pool_authority: &'a AccountInfo<'b>,
    liquidity_mint: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    user_token_account: &'a AccountInfo<'b>,
    user: &'a AccountInfo<'b>,
    clock: &'a AccountInfo<'b>,
}

impl<'a, 'b> WithdrawMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<WithdrawMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();
        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_pool_spl = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let reward_pool_authority = AccountLoader::next_uninitialized(account_info_iter)?;
        let liquidity_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let user_token_account =
            AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let user = AccountLoader::next_signer(account_info_iter)?;

        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;
        let clock = AccountLoader::next_with_key(account_info_iter, &clock::id())?;

        Ok(WithdrawMiningContext {
            reward_pool,
            reward_pool_spl,
            reward_pool_authority,
            liquidity_mint,
            mining,
            user_token_account,
            user,
            clock,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        let mining = Mining::unpack(&self.mining.data.borrow())?;

        let reward_pool_seeds = &[
            b"reward_pool".as_ref(),
            &reward_pool.rewards_root.to_bytes()[..32],
            &reward_pool.liquidity_mint.to_bytes()[..32],
            &[reward_pool.bump],
        ];

        {
            assert_account_key(self.user, &mining.owner)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            assert_account_key(
                self.reward_pool,
                &Pubkey::create_program_address(reward_pool_seeds, program_id)?,
            )?;
        }

        {
            let mining_pubkey = Pubkey::create_program_address(
                &[
                    b"mining".as_ref(),
                    self.user.key.as_ref(),
                    self.reward_pool.key.as_ref(),
                    &[mining.bump],
                ],
                program_id,
            )?;
            assert_account_key(self.mining, &mining_pubkey)?;
        }

        {
            let (spl_pubkey, _) = find_reward_pool_spl_token_account(
                program_id,
                self.reward_pool.key,
                self.liquidity_mint.key,
            );

            assert_account_key(self.reward_pool_spl, &spl_pubkey)?;
        }

        // check if it's allowed to withdraw
        let timestamp = Clock::from_account_info(self.clock)?.unix_timestamp as u64;
        if timestamp.saturating_sub(mining.last_deposit_time) < reward_pool.lock_time_sec {
            return Err(EverlendError::LockTimeStillActive.into());
        }

        reward_pool.withdraw(mining.amount)?;
        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        self.spl_transfer_and_close(program_id, mining.amount)?;

        Ok(())
    }

    fn spl_transfer_and_close(&self, program_id: &Pubkey, amount: u64) -> ProgramResult {
        let (reward_pool_authority, bump_seed) =
            find_program_address(program_id, self.reward_pool.key);
        assert_account_key(self.reward_pool_authority, &reward_pool_authority)?;
        let signers_seeds = &[self.reward_pool.key.as_ref(), &[bump_seed]];

        // Transfer token from source to token account
        everlend_utils::cpi::spl_token::transfer(
            self.reward_pool_spl.clone(),
            self.user_token_account.clone(),
            self.reward_pool_authority.clone(),
            amount,
            &[signers_seeds],
        )?;

        // close mining account
        everlend_utils::cpi::system::close_account(self.mining, self.user)?;

        Ok(())
    }
}
