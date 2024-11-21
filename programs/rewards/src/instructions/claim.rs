use crate::state::{Mining, RewardPool};
use everlend_utils::{assert_account_key, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::{Sysvar, SysvarId};
use spl_token::state::Account;

/// Instruction context
pub struct ClaimContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    vault: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    user: &'a AccountInfo<'b>,
    user_reward_token_account: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
}

impl<'a, 'b> ClaimContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<ClaimContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let vault = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let user = AccountLoader::next_signer(account_info_iter)?;
        let user_reward_token_account = AccountLoader::next_unchecked(account_info_iter)?; // unchecked so we can create on the fly
        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;
        let rent = AccountLoader::next_with_key(account_info_iter, &Rent::id())?;

        Ok(ClaimContext {
            reward_pool,
            reward_mint,
            vault,
            mining,
            user,
            user_reward_token_account,
            rent,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        let mut mining = Mining::unpack(&self.mining.data.borrow())?;

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

            let bump = reward_pool
                .vaults
                .iter()
                .find(|v| &v.reward_mint == self.reward_mint.key)
                .ok_or(ProgramError::InvalidArgument)?
                .bump;
            let vault_seeds = &[
                b"vault".as_ref(),
                &self.reward_pool.key.to_bytes()[..32],
                &self.reward_mint.key.to_bytes()[..32],
                &[bump],
            ];
            assert_account_key(
                self.vault,
                &Pubkey::create_program_address(vault_seeds, program_id)?,
            )?;
        }

        mining.refresh_rewards(reward_pool.vaults.iter())?;

        let reward_index = mining.reward_index_mut(*self.reward_mint.key);
        let amount = reward_index.rewards;

        reward_index.rewards = 0;

        // create user token account if it does not exist
        if self.user_reward_token_account.owner.eq(&Pubkey::default()) {
            everlend_utils::cpi::system::create_account::<Account>(
                &spl_token::id(),
                self.user.clone(),
                self.user_reward_token_account.clone(),
                &[],
                &Rent::from_account_info(self.rent)?,
            )?;

            everlend_utils::cpi::spl_token::initialize_account(
                self.user_reward_token_account.clone(),
                self.reward_mint.clone(),
                self.user.clone(),
                self.rent.clone(),
            )?;
        }

        everlend_utils::cpi::spl_token::transfer(
            self.vault.clone(),
            self.user_reward_token_account.clone(),
            self.reward_pool.clone(),
            amount,
            &[reward_pool_seeds],
        )?;

        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        Ok(())
    }
}
