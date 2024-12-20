use crate::state::{AccountType, Mining};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use everlend_utils::EverlendError;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;
use crate::state::deprecated_reward_pool::DeprecatedRewardPool;

/// Max reward vaults
pub const MAX_REWARDS: usize = 3;
/// Max reward tiers
pub const MAX_TIERS: usize = 5;

/// Reward pool
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct RewardPool {
    /// Account type - RewardPool
    pub account_type: AccountType,
    /// Rewards root account (ex-Config program account)
    pub rewards_root: Pubkey,
    /// Saved bump for reward pool account
    pub bump: u8,
    /// Liquidity mint
    pub liquidity_mint: Pubkey,
    /// max stakers
    pub max_stakers: u64,
    /// max stakers
    pub total_stakers: u64,
    /// Total staked amount
    pub total_amount: u64,
    /// staking lock time
    pub lock_time_sec: u64,
    /// A set of all possible rewards that we can get for this pool
    pub vaults: Vec<RewardVault>,
}

impl RewardPool {
    /// Init reward pool
    pub fn init(params: InitRewardPoolParams) -> RewardPool {
        RewardPool {
            account_type: AccountType::RewardPool,
            rewards_root: params.rewards_root,
            bump: params.bump,
            liquidity_mint: params.liquidity_mint,
            total_amount: 0,
            total_stakers: 0,
            lock_time_sec: params.lock_time_sec,
            vaults: vec![],
            max_stakers: params.max_stakers,
        }
    }

    /// Returns reward index
    pub fn update_vault_totals(&mut self, reward_mint: Pubkey, amount: u64) -> ProgramResult {
        match self
            .vaults
            .iter()
            .position(|mi| mi.reward_mint == reward_mint)
        {
            Some(i) => {
                self.vaults[i].claimed_total_amount += amount;
                Ok(())
            }
            None => Err(EverlendError::InvalidRewardVault.into()),
        }
    }

    /// Process add vault
    pub fn add_vault(&mut self, reward: RewardVault) -> ProgramResult {
        if self
            .vaults
            .iter()
            .any(|v| v.reward_mint == reward.reward_mint)
        {
            return Err(ProgramError::InvalidArgument);
        }

        if self.vaults.len() == MAX_REWARDS {
            return Err(EverlendError::InvalidRewardVault.into());
        }

        self.vaults.push(reward);

        Ok(())
    }

    /// Process deposit
    pub fn deposit(
        &mut self,
        mining: &mut Mining,
        amount: u64,
        is_first_deposit: bool,
        timestamp: u64,
    ) -> ProgramResult {
        mining.refresh_rewards(self.vaults.iter(), timestamp)?;

        if is_first_deposit {
            if self.max_stakers > 0 && self.total_stakers >= self.max_stakers {
                return Err(EverlendError::PoolIsFull.into());
            }

            self.total_stakers += 1;
        }

        self.total_amount = self
            .total_amount
            .checked_add(amount)
            .ok_or(EverlendError::MathOverflow)?;

        mining.amount = mining
            .amount
            .checked_add(amount)
            .ok_or(EverlendError::MathOverflow)?;

        mining.last_deposit_time = timestamp;

        Ok(())
    }

    /// Process withdraw
    pub fn withdraw(&mut self, amount: u64) -> ProgramResult {
        self.total_amount = self
            .total_amount
            .checked_sub(amount)
            .ok_or(EverlendError::MathOverflow)?;

        self.total_stakers -= 1;

        Ok(())
    }

    /// Process migrate
    pub fn migrate(deprecated_pool: &DeprecatedRewardPool, max_stakers: u64, total_stakers: u64) -> RewardPool {
        Self {
            account_type: deprecated_pool.account_type.clone(),
            rewards_root: deprecated_pool.rewards_root,
            bump: deprecated_pool.bump,
            liquidity_mint: deprecated_pool.liquidity_mint,
            max_stakers,
            total_stakers,
            total_amount: deprecated_pool.total_amount,
            lock_time_sec: deprecated_pool.lock_time_sec,
            vaults: deprecated_pool.vaults.iter().map(|v| RewardVault{
                vault_token_account_bump: v.vault_token_account_bump,
                reward_mint: v.reward_mint,
                reward_period_sec: v.reward_period_sec,
                is_enabled: v.is_enabled,
                enabled_at: v.enabled_at,
                claimed_total_amount: 0,
                reward_tiers: v.reward_tiers.clone(),
            }).collect(),
        }
    }
}

/// Initialize a Reward Pool params
pub struct InitRewardPoolParams {
    /// Rewards Root
    pub rewards_root: Pubkey,
    /// Saved bump for reward pool account
    pub bump: u8,
    /// Liquidity mint
    pub liquidity_mint: Pubkey,
    /// staking lock time
    pub lock_time_sec: u64,
    /// max stakers
    pub max_stakers: u64,
}

impl Sealed for RewardPool {}
impl Pack for RewardPool {
    const LEN: usize = 1 + (32 + 1 + 32 + 8 + 8 + 8 + 8 + (4 + RewardVault::LEN * MAX_REWARDS));

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<RewardPool, ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|err| {
            msg!("Failed to deserialize");
            msg!("{}", err.to_string());
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for RewardPool {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::RewardPool
    }
}

/// Reward vault
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone)]
pub struct RewardVault {
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
    /// Total rewards
    pub claimed_total_amount: u64,
    /// Reward tiers
    pub reward_tiers: Vec<RewardTier>,
}

impl RewardVault {
    /// LEN
    pub const LEN: usize = 1 + 32 + 4 + 1 + 8 + 8 + (4 + RewardTier::LEN * MAX_TIERS);
}

/// Reward vault
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq, Eq, Clone)]
pub struct RewardTier {
    /// Reward ratio of deposit currency
    pub ratio_base: u64,
    /// Reward ratio of reward currency
    pub ratio_quote: u64,
    /// Maximum amount of reward per period (cap)
    pub reward_max_amount_per_period: u64,
}

impl RewardTier {
    /// LEN
    pub const LEN: usize = 8 + 8 + 8;
}
