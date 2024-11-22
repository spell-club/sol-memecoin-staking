use crate::state::{RewardVault, MAX_REWARDS};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use everlend_utils::EverlendError;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;
use std::cmp;
use std::ops::Div;
use std::slice::Iter;

use super::AccountType;

/// Mining
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Mining {
    /// Account type - Mining
    pub account_type: AccountType,
    /// Reward pool address
    pub reward_pool: Pubkey,
    /// Saved bump for mining account
    pub bump: u8,
    /// Amount of staked
    pub amount: u64,
    /// Last rewards calculation
    pub rewards_calculated_at: u64,
    /// Mining owner
    pub owner: Pubkey,
    /// last deposit time
    pub last_deposit_time: u64,
    /// Reward indexes
    pub indexes: Vec<RewardIndex>,
}

impl Mining {
    /// Initialize a Reward Pool
    pub fn initialize(reward_pool: Pubkey, bump: u8, owner: Pubkey) -> Mining {
        Mining {
            account_type: AccountType::Mining,
            reward_pool,
            bump,
            amount: 0,
            rewards_calculated_at: 0,
            last_deposit_time: 0,
            owner,
            indexes: vec![],
        }
    }

    /// Returns reward index
    pub fn reward_index_mut(&mut self, reward_mint: Pubkey) -> &mut RewardIndex {
        match self
            .indexes
            .iter()
            .position(|mi| mi.reward_mint == reward_mint)
        {
            Some(i) => &mut self.indexes[i],
            None => {
                self.indexes.push(RewardIndex {
                    reward_mint,
                    ..Default::default()
                });
                self.indexes.last_mut().unwrap()
            }
        }
    }

    /// Claim reward
    pub fn claim(&mut self, reward_mint: Pubkey) {
        let reward_index = self.reward_index_mut(reward_mint);
        reward_index.rewards = 0;
    }

    /// Refresh rewards
    pub fn refresh_rewards(
        &mut self,
        vaults: Iter<RewardVault>,
        current_timestamp: u64,
    ) -> ProgramResult {
        let rewards_calculated_at = self.rewards_calculated_at;

        // first deposit - nothing to calculate
        if rewards_calculated_at != 0 {
            let share = self.amount;

            for vault in vaults {
                let reward_index = self.reward_index_mut(vault.reward_mint);

                // how much time passed since last reward calculation
                let reward_period_start =
                    cmp::max(rewards_calculated_at, vault.distribution_starts_at);
                let reward_period = current_timestamp.saturating_sub(reward_period_start);
                let num_periods = reward_period.div(vault.reward_period_sec as u64);
                if num_periods == 0 {
                    continue;
                }

                // calculate reward amount based on coefficient
                let rewards = (num_periods as u128)
                    .checked_mul(share.into())
                    .ok_or(EverlendError::MathOverflow)?
                    .checked_mul(vault.ratio_quote.into())
                    .ok_or(EverlendError::MathOverflow)?
                    .checked_div(vault.ratio_base.into())
                    .ok_or(EverlendError::MathOverflow)? as u64;

                if rewards > 0 {
                    let rewards = if vault.reward_max_amount_per_period > 0 {
                        std::cmp::min(rewards, vault.reward_max_amount_per_period * num_periods)
                    } else {
                        rewards
                    };

                    reward_index.rewards = reward_index
                        .rewards
                        .checked_add(rewards as u64)
                        .ok_or(EverlendError::MathOverflow)?;
                }
            }
        }

        // update deposit_timestamp
        self.rewards_calculated_at = current_timestamp;

        Ok(())
    }
}

impl Sealed for Mining {}
impl Pack for Mining {
    const LEN: usize = 1 + (32 + 1 + 8 + 8 + 32 + 8 + (4 + RewardIndex::LEN * MAX_REWARDS));

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|err| {
            msg!("Failed to deserialize");
            msg!("{}", err.to_string());
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for Mining {
    fn is_initialized(&self) -> bool {
        self.owner != Pubkey::default()
    }
}

/// Reward index
#[derive(Debug, BorshSerialize, BorshDeserialize, BorshSchema, Default, Clone)]
pub struct RewardIndex {
    /// Reward mint
    pub reward_mint: Pubkey,
    /// Rewards amount
    pub rewards: u64,
}

impl RewardIndex {
    ///
    pub const LEN: usize = 32 + 8;
}
