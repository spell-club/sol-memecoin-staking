use crate::find_vault_program_address;
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
use spl_token::state::Account;

use crate::state::{RewardPool, RewardTier, RewardVault, RewardsRoot};

/// Instruction context
pub struct AddVaultContext<'a, 'b> {
    rewards_root: &'a AccountInfo<'b>,
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    vault: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    clock: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
}

impl<'a, 'b> AddVaultContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<AddVaultContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let rewards_root = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let vault = AccountLoader::next_uninitialized(account_info_iter)?;
        let authority = AccountLoader::next_signer(account_info_iter)?;
        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;
        let clock = AccountLoader::next_with_key(account_info_iter, &clock::id())?;
        let rent = AccountLoader::next_with_key(account_info_iter, &Rent::id())?;

        Ok(AddVaultContext {
            rewards_root,
            reward_pool,
            reward_mint,
            vault,
            payer: authority,
            clock,
            rent,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        program_id: &Pubkey,
        reward_period_sec: u32,
        is_enabled: bool,
        reward_tiers: Vec<RewardTier>,
    ) -> ProgramResult {
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        assert_account_key(self.rewards_root, &reward_pool.rewards_root)?;

        {
            let rewards_root = RewardsRoot::unpack(&self.rewards_root.data.borrow())?;
            assert_account_key(self.payer, &rewards_root.authority)?;
        }

        let timestamp = Clock::from_account_info(self.clock)?.unix_timestamp;
        let bump = self.create_spl_acc(program_id)?;

        reward_pool.add_vault(RewardVault {
            bump,
            reward_period_sec,
            reward_mint: *self.reward_mint.key,
            is_enabled,
            enabled_at: if is_enabled { timestamp as u64 } else { 0 },
            reward_tiers,
        })?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }

    /// creates vault spl token account
    pub fn create_spl_acc(&self, program_id: &Pubkey) -> Result<u8, ProgramError> {
        let bump = {
            // I guess better to move this general check into process func 
            let (vault_pubkey, bump) =
                find_vault_program_address(program_id, self.reward_pool.key, self.reward_mint.key);
            assert_account_key(self.vault, &vault_pubkey)?;

            bump
        };

        let signers_seeds = &[
            b"vault".as_ref(),
            self.reward_pool.key.as_ref(),
            self.reward_mint.key.as_ref(),
            &[bump],
        ];

        everlend_utils::cpi::system::create_account::<Account>(
            &spl_token::id(),
            self.payer.clone(),
            self.vault.clone(),
            &[signers_seeds],
            &Rent::from_account_info(self.rent)?,
        )?;

        everlend_utils::cpi::spl_token::initialize_account(
            self.vault.clone(),
            self.reward_mint.clone(),
            self.reward_pool.clone(),
            self.rent.clone(),
        )?;

        Ok(bump)
    }
}
