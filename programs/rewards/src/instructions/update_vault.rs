use everlend_utils::{assert_account_key, AccountLoader};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use solana_program::{account_info::AccountInfo, sysvar::clock};

use crate::state::{RewardPool, RewardTier, RewardsRoot};

/// Instruction context
pub struct UpdateVaultContext<'a, 'b> {
    rewards_root: &'a AccountInfo<'b>,
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    clock: &'a AccountInfo<'b>,
}

impl<'a, 'b> UpdateVaultContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<UpdateVaultContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let rewards_root = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let payer = AccountLoader::next_signer(account_info_iter)?;
        let clock = AccountLoader::next_with_key(account_info_iter, &clock::id())?;

        Ok(UpdateVaultContext {
            rewards_root,
            reward_pool,
            reward_mint,
            payer,
            clock,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        _program_id: &Pubkey,
        reward_period_sec: Option<u32>,
        is_enabled: Option<bool>,
        reward_tiers: Option<Vec<RewardTier>>,
    ) -> ProgramResult {
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        assert_account_key(self.rewards_root, &reward_pool.rewards_root)?;

        {
            let rewards_root = RewardsRoot::unpack(&self.rewards_root.data.borrow())?;
            assert_account_key(self.payer, &rewards_root.authority)?;
        }

        let vault = reward_pool
            .vaults
            .iter_mut()
            .find(|v| &v.reward_mint == self.reward_mint.key)
            .ok_or(ProgramError::InvalidArgument)?;

        if let Some(reward_period_sec) = reward_period_sec {
            vault.reward_period_sec = reward_period_sec;
        }

        if let Some(is_enabled) = is_enabled {
            if !vault.is_enabled && is_enabled {
                // enabling vault - update the time
                vault.enabled_at = Clock::from_account_info(self.clock)?.unix_timestamp as u64;
            }

            vault.is_enabled = is_enabled;
        }

        if let Some(reward_tiers) = reward_tiers {
            vault.reward_tiers = reward_tiers;
        }

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
