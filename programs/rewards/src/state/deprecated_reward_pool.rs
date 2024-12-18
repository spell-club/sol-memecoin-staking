use crate::state::{AccountType, InitRewardPoolParams, RewardTier, MAX_REWARDS, MAX_TIERS};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

/// Deprecated Reward pool
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct DeprecatedRewardPool {
    /// Account type - RewardPool
    pub account_type: AccountType,
    /// Rewards root account (ex-Config program account)
    pub rewards_root: Pubkey,
    /// Saved bump for reward pool account
    pub bump: u8,
    /// Liquidity mint
    pub liquidity_mint: Pubkey,
    /// Total staked amount
    pub total_amount: u64,
    /// staking lock time
    pub lock_time_sec: u64,
    /// A set of all possible rewards that we can get for this pool
    pub vaults: Vec<DeprecatedRewardVault>,
}

impl Sealed for DeprecatedRewardPool {}
impl Pack for DeprecatedRewardPool {
    const LEN: usize = 8 + (32 + 1 + 32 + 8 + (4 + DeprecatedRewardVault::LEN * MAX_REWARDS) + 32);

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<DeprecatedRewardPool, ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|err| {
            msg!("Failed to deserialize");
            msg!("{}", err.to_string());
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for DeprecatedRewardPool {
    fn is_initialized(&self) -> bool {
        self.rewards_root != Pubkey::default()
    }
}

impl DeprecatedRewardPool {
    /// Init deprecated reward pool
    pub fn init(params: InitRewardPoolParams) -> DeprecatedRewardPool {
        DeprecatedRewardPool {
            account_type: AccountType::RewardPool,
            rewards_root: params.rewards_root,
            bump: params.bump,
            liquidity_mint: params.liquidity_mint,
            total_amount: 0,
            lock_time_sec: params.lock_time_sec,
            vaults: vec![],
        }
    }
}


/// Reward vault
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone)]
pub struct DeprecatedRewardVault {
    /// Bump of vault account
    pub vault_token_account_bump: u8,
    /// Reward mint address
    pub reward_mint: Pubkey,
    /// Time period for reward calculation
    pub reward_period_sec: u32,
    /// Is distribution enabled
    pub is_enabled: bool,
    /// Timestamp since when distribution begins
    pub enabled_at: u64,
    /// Reward tiers
    pub reward_tiers: Vec<RewardTier>,
}

impl DeprecatedRewardVault {
    /// LEN
    pub const LEN: usize = 1 + 32 + 4 + 1 + 8 + (4 + RewardTier::LEN * MAX_TIERS);
}
