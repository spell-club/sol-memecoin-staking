use crate::find_reward_pool_spl_program_address;
use crate::state::{Mining, RewardPool};
use everlend_utils::{assert_account_key, find_program_address, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;

/// Instruction context
pub struct WithdrawMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_pool_spl: &'a AccountInfo<'b>,
    reward_pool_authority: &'a AccountInfo<'b>,
    liquidity_mint: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    user_token_account: &'a AccountInfo<'b>,
    user: &'a AccountInfo<'b>,
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

        Ok(WithdrawMiningContext {
            reward_pool,
            reward_pool_spl,
            reward_pool_authority,
            liquidity_mint,
            mining,
            user_token_account,
            user,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        let mining = Mining::unpack(&self.mining.data.borrow())?;

        // TODO: make sure it's users mining account
        // TDO: make sure user receives spl from proper pool

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
            // assert_account_key(self.deposit_authority, &reward_pool.deposit_authority)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            assert_account_key(self.user, &mining.owner)?;
        }

        {
            let (spl_pubkey, _) = find_reward_pool_spl_program_address(
                program_id,
                self.reward_pool.key,
                self.liquidity_mint.key,
            );

            assert_account_key(self.reward_pool_spl, &spl_pubkey)?;
        }

        let (reward_pool_authority, bump_seed) =
            find_program_address(program_id, self.reward_pool.key);
        assert_account_key(self.reward_pool_authority, &reward_pool_authority)?;
        let signers_seeds = &[self.reward_pool.key.as_ref(), &[bump_seed]];

        // Transfer token from source to token account
        everlend_utils::cpi::spl_token::transfer(
            self.reward_pool_spl.clone(),
            self.user_token_account.clone(),
            self.reward_pool_authority.clone(),
            mining.share,
            &[signers_seeds],
        )?;

        reward_pool.withdraw(mining.share)?;
        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        // close mining account
        everlend_utils::cpi::system::close_account(self.mining, self.user)?;

        Ok(())
    }
}
