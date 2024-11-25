use crate::state::{Mining, RewardPool};
use crate::{find_mining_program_address, find_reward_pool_spl_program_address};
use everlend_utils::{assert_account_key, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::{clock, Sysvar, SysvarId};

/// Instruction context
pub struct DepositMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_pool_spl: &'a AccountInfo<'b>,
    liquidity_mint: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    user_token_account: &'a AccountInfo<'b>,
    user: &'a AccountInfo<'b>,
    clock: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
}

impl<'a, 'b> DepositMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<DepositMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();
        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_pool_spl = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let liquidity_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let mining = AccountLoader::next_unchecked(account_info_iter)?; // unchecked so we can create on the fly
        let user_token_account =
            AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let user = AccountLoader::next_signer(account_info_iter)?;

        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;
        let clock = AccountLoader::next_with_key(account_info_iter, &clock::id())?;
        let rent = AccountLoader::next_with_key(account_info_iter, &Rent::id())?;

        Ok(DepositMiningContext {
            reward_pool,
            reward_pool_spl,
            liquidity_mint,
            mining,
            user_token_account,
            user,
            clock,
            rent,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey, amount: u64) -> ProgramResult {
        let mut mining = self.check_and_init_mining(program_id)?;
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
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            assert_account_key(self.user, &mining.owner)?;
        }

        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        {
            let (spl_pubkey, _) = find_reward_pool_spl_program_address(
                program_id,
                self.reward_pool.key,
                self.liquidity_mint.key,
            );

            assert_account_key(self.reward_pool_spl, &spl_pubkey)?;
        }

        let timestamp = Clock::from_account_info(self.clock)?.unix_timestamp;

        // Transfer token from source to token account
        everlend_utils::cpi::spl_token::transfer(
            self.user_token_account.clone(),
            self.reward_pool_spl.clone(),
            self.user.clone(),
            amount,
            &[],
        )?;

        reward_pool.deposit(&mut mining, amount, timestamp as u64)?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;
        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        Ok(())
    }

    /// Process instruction
    pub fn check_and_init_mining(&self, program_id: &Pubkey) -> Result<Mining, ProgramError> {
        if self.mining.owner.eq(&Pubkey::default()) {
            // create account
            let bump = self.create_mining_acc(program_id)?;
            return Ok(Mining::initialize(
                *self.reward_pool.key,
                bump,
                *self.user.key,
            ));
        }

        if self.mining.owner.eq(program_id) {
            return Ok(Mining::unpack(&self.mining.data.borrow())?);
        }

        Err(ProgramError::InvalidAccountOwner)
    }

    /// create a mining account for user
    pub fn create_mining_acc(&self, program_id: &Pubkey) -> Result<u8, ProgramError> {
        let bump = {
            let (pubkey, bump) =
                find_mining_program_address(program_id, self.user.key, self.reward_pool.key);
            assert_account_key(self.mining, &pubkey)?;
            bump
        };

        let signers_seeds = &[
            "mining".as_bytes(),
            &self.user.key.to_bytes(),
            &self.reward_pool.key.to_bytes(),
            &[bump],
        ];

        everlend_utils::cpi::system::create_account::<Mining>(
            program_id,
            self.user.clone(),
            self.mining.clone(),
            &[signers_seeds],
            &Rent::from_account_info(self.rent)?,
        )?;

        Ok(bump)
    }
}
