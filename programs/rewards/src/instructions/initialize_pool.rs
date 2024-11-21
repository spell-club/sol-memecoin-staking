use crate::state::{InitRewardPoolParams, RewardPool, RewardsRoot};
use crate::{find_reward_pool_program_address, find_reward_pool_spl_program_address};
use everlend_utils::{assert_account_key, find_program_address, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::{Sysvar, SysvarId};
use spl_token::state::Account;

/// Instruction context
pub struct InitializePoolContext<'a, 'b> {
    rewards_root: &'a AccountInfo<'b>,
    reward_pool: &'a AccountInfo<'b>,
    reward_pool_spl: &'a AccountInfo<'b>,
    reward_pool_authority: &'a AccountInfo<'b>,
    liquidity_mint: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
}

impl<'a, 'b> InitializePoolContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<InitializePoolContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let rewards_root = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_pool = AccountLoader::next_uninitialized(account_info_iter)?;
        let reward_pool_spl = AccountLoader::next_uninitialized(account_info_iter)?;
        let reward_pool_authority = AccountLoader::next_uninitialized(account_info_iter)?;
        let liquidity_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let payer = AccountLoader::next_signer(account_info_iter)?;
        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;
        let rent = AccountLoader::next_with_key(account_info_iter, &Rent::id())?;

        Ok(InitializePoolContext {
            rewards_root,
            reward_pool,
            reward_pool_spl,
            reward_pool_authority,
            liquidity_mint,
            payer,
            rent,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        {
            let rewards_root = RewardsRoot::unpack(&self.rewards_root.data.borrow())?;
            assert_account_key(self.payer, &rewards_root.authority)?;
        }

        self.create_spl_acc(program_id)?;
        self.create_rewards_pool_acc(program_id)?;

        Ok(())
    }

    /// create pool account
    pub fn create_spl_acc(&self, program_id: &Pubkey) -> ProgramResult {
        {
            let bump = {
                let (spl_pubkey, bump) = find_reward_pool_spl_program_address(
                    program_id,
                    self.reward_pool.key,
                    self.liquidity_mint.key,
                );
                assert_account_key(self.reward_pool_spl, &spl_pubkey)?;

                bump
            };

            let signers_seeds = &[
                b"spl".as_ref(),
                self.reward_pool.key.as_ref(),
                self.liquidity_mint.key.as_ref(),
                &[bump],
            ];

            everlend_utils::cpi::system::create_account::<Account>(
                &spl_token::id(),
                self.payer.clone(),
                self.reward_pool_spl.clone(),
                &[signers_seeds],
                &Rent::from_account_info(self.rent)?,
            )?;
        }

        let (reward_pool_authority, bump_seed) =
            find_program_address(program_id, self.reward_pool.key);
        assert_account_key(self.reward_pool_authority, &reward_pool_authority)?;

        everlend_utils::cpi::spl_token::initialize_account(
            self.reward_pool_spl.clone(),
            self.liquidity_mint.clone(),
            self.reward_pool_authority.clone(),
            self.rent.clone(),
        )?;

        Ok(())
    }

    /// create pool account
    pub fn create_rewards_pool_acc(&self, program_id: &Pubkey) -> ProgramResult {
        let bump = {
            let (reward_pool_pubkey, bump) = find_reward_pool_program_address(
                program_id,
                self.rewards_root.key,
                self.liquidity_mint.key,
            );
            assert_account_key(self.reward_pool, &reward_pool_pubkey)?;
            bump
        };

        let reward_pool_seeds = &[
            "reward_pool".as_bytes(),
            self.rewards_root.key.as_ref(),
            self.liquidity_mint.key.as_ref(),
            &[bump],
        ];

        everlend_utils::cpi::system::create_account::<RewardPool>(
            program_id,
            self.payer.clone(),
            self.reward_pool.clone(),
            &[reward_pool_seeds],
            &Rent::from_account_info(self.rent)?,
        )?;

        let reward_pool = RewardPool::init(InitRewardPoolParams {
            rewards_root: *self.rewards_root.key,
            bump,
            liquidity_mint: *self.liquidity_mint.key,
        });

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
