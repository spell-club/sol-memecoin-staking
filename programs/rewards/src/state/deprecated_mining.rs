use crate::state::{AccountType, MAX_REWARDS};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

/// Deprecated Mining
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct DeprecatedMining {
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
    pub indexes: Vec<DeprecatedRewardIndex>,
}


impl Sealed for DeprecatedMining {}
impl Pack for DeprecatedMining {
    const LEN: usize = 1 + (32 + 1 + 8 + 8 + 32 + 8 + 1 + (4 + DeprecatedRewardIndex::LEN * MAX_REWARDS));

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

impl IsInitialized for DeprecatedMining {
    fn is_initialized(&self) -> bool {
        self.owner != Pubkey::default()
    }
}

/// Deprecated Reward index
#[derive(Debug, BorshSerialize, BorshDeserialize, BorshSchema, Default, Clone)]
pub struct DeprecatedRewardIndex {
    /// Reward mint
    pub reward_mint: Pubkey,
    /// Rewards amount
    pub rewards: u64,
}

impl DeprecatedRewardIndex {
    ///
    pub const LEN: usize = 32 + 8;
}