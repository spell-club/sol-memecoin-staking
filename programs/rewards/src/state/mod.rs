//! State types

mod mining;
mod reward_pool;
mod rewards_root;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
pub use mining::*;
pub use reward_pool::*;
pub use rewards_root::*;

/// Enum representing the account type managed by the program
#[derive(Default, Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum AccountType {
    /// If the account has not been initialized, the enum will be 0
    #[default]
    Uninitialized,
    /// Rewards root
    RewardsRoot,
    /// Reward pool
    RewardPool,
    /// Mining account
    Mining,
}
