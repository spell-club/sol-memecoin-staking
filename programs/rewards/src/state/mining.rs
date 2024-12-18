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
use crate::state::deprecated_mining::DeprecatedMining;
use super::AccountType;

/// Mining
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
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
    /// reward tier
    pub reward_tier: u8,
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
            reward_tier: 0,
            indexes: vec![],
        }
    }

    /// Process migrate
    pub fn migrate(deprecated_mining: &DeprecatedMining) -> Mining {
        Self {
            account_type: deprecated_mining.account_type.clone(),
            reward_pool: deprecated_mining.reward_pool,
            bump: deprecated_mining.bump,
            amount: deprecated_mining.amount,
            rewards_calculated_at: deprecated_mining.rewards_calculated_at,
            owner: deprecated_mining.owner,
            last_deposit_time: deprecated_mining.last_deposit_time,
            reward_tier: deprecated_mining.reward_tier,
            indexes: deprecated_mining.indexes.iter().map(|i| RewardIndex{
                reward_mint: i.reward_mint,
                rewards: i.rewards,
                claimed_total_rewards: 0,
            }).collect(),
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

    /// Flush rewards
    pub fn flush_rewards(&mut self, reward_mint: Pubkey) -> u64 {
        let reward_index = self.reward_index_mut(reward_mint);
        let amount = reward_index.rewards;
        reward_index.rewards = 0;
        reward_index.claimed_total_rewards += amount;

        amount
    }

    /// Refresh rewards
    pub fn refresh_rewards(
        &mut self,
        vaults: Iter<RewardVault>,
        current_timestamp: u64,
    ) -> ProgramResult {
        let rewards_calculated_at = self.rewards_calculated_at;
        let rewards_tier = self.reward_tier as usize;

        // first deposit - nothing to calculate
        if rewards_calculated_at != 0 {
            let amount = self.amount;

            for vault in vaults {
                if !vault.is_enabled {
                    continue;
                }

                let reward_index = self.reward_index_mut(vault.reward_mint);

                // how much time passed since last reward calculation
                let reward_period_start = cmp::max(rewards_calculated_at, vault.enabled_at);
                let reward_period = current_timestamp.saturating_sub(reward_period_start);
                let num_periods = reward_period.div(vault.reward_period_sec as u64);
                if num_periods == 0 {
                    continue;
                }

                // get proper reward tier idx
                let tier_idx = cmp::min(rewards_tier, vault.reward_tiers.len().saturating_sub(1));

                let tier = vault
                    .reward_tiers
                    .get(tier_idx)
                    .ok_or(EverlendError::InvalidRewardTier)?;

                // calculate reward amount based on coefficient
                let rewards = (num_periods as u128)
                    .checked_mul(amount.into())
                    .ok_or(EverlendError::MathOverflow)?
                    .checked_mul(tier.ratio_quote.into())
                    .ok_or(EverlendError::MathOverflow)?
                    .checked_div(tier.ratio_base.into())
                    .ok_or(EverlendError::MathOverflow)? as u64;

                if rewards > 0 {
                    let rewards = if tier.reward_max_amount_per_period > 0 {
                        std::cmp::min(rewards, tier.reward_max_amount_per_period * num_periods)
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

        // update rewards calculation timestamp
        self.rewards_calculated_at = current_timestamp;

        Ok(())
    }
}

impl Sealed for Mining {}
impl Pack for Mining {
    const LEN: usize = 1 + (32 + 1 + 8 + 8 + 32 + 8 + 1 + (4 + RewardIndex::LEN * MAX_REWARDS));

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
    /// claimed_total_rewards
    pub claimed_total_rewards: u64,
}

impl RewardIndex {
    ///
    pub const LEN: usize = 32 + 8 + 8;
}
