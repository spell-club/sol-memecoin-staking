//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::clock;
use solana_program::{system_program, sysvar};

use crate::state::RewardTier;

/// Instructions supported by the program
#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
pub enum RewardsInstruction {
    /// Creates and initializes a reward pool account
    InitializePool {
        /// staking lock time
        lock_time_sec: u64,
    },

    /// Creates a new vault account and adds it to the reward pool
    AddVault {
        /// Time period for reward calculation
        reward_period_sec: u32,
        /// Is distribution enabled
        is_enabled: bool,
        /// reward tiers
        tiers: Vec<RewardTier>,
    },

    /// Updates vault parameters
    UpdateVault {
        /// Time period for reward calculation
        reward_period_sec: Option<u32>,
        /// Is distribution enabled
        is_enabled: Option<bool>,
        /// reward tiers
        tiers: Option<Vec<RewardTier>>,
    },

    /// Fills the reward pool with rewards
    FillVault {
        /// Amount to fill
        amount: u64,
    },

    /// Deposits amount of supply to the mining account
    DepositMining {
        /// Amount to deposit
        amount: u64,
    },

    /// Withdraws amount of supply to the mining account
    WithdrawMining,

    /// Claims amount of rewards
    Claim,

    /// Upgrades mining account for users (admin method)
    UpgradeMining {
        /// set user reward tier
        tier: u8,
    },

    /// Creates and initializes a reward root
    InitializeRoot,

    /// Migrates reward pool
    MigratePool,
}

/// Creates 'InitializePool' instruction.
pub fn initialize_pool(
    program_id: &Pubkey,
    root_account: &Pubkey,
    reward_pool: &Pubkey,
    reward_pool_spl: &Pubkey,
    reward_pool_authority: &Pubkey,
    liquidity_mint: &Pubkey,
    payer: &Pubkey,
    lock_time_sec: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*root_account, false),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*reward_pool_spl, false),
        AccountMeta::new_readonly(*reward_pool_authority, false),
        AccountMeta::new_readonly(*liquidity_mint, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::InitializePool { lock_time_sec },
        accounts,
    )
}

/// Creates 'AddVault' instruction.
#[allow(clippy::too_many_arguments)]
pub fn add_vault(
    program_id: &Pubkey,
    rewards_root: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    payer: &Pubkey,
    reward_period_sec: u32,
    tiers: Vec<RewardTier>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*rewards_root, false),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::AddVault {
            reward_period_sec,
            is_enabled: true,
            tiers,
        },
        accounts,
    )
}

/// Creates 'UpdateVault' instruction.
#[allow(clippy::too_many_arguments)]
pub fn update_vault(
    program_id: &Pubkey,
    rewards_root: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    payer: &Pubkey,
    reward_period_sec: Option<u32>,
    is_enabled: Option<bool>,
    tiers: Option<Vec<RewardTier>>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*rewards_root, false),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(clock::id(), false),
    ];
    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::UpdateVault {
            reward_period_sec,
            is_enabled,
            tiers,
        },
        accounts,
    )
}
/// Creates 'FillVault' instruction.
#[allow(clippy::too_many_arguments)]
pub fn fill_vault(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    from: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*from, false),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::FillVault { amount },
        accounts,
    )
}

/// Creates 'DepositMining' instruction.
#[allow(clippy::too_many_arguments)]
pub fn deposit_mining(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_pool_spl: &Pubkey,
    liquidity_mint: &Pubkey,
    mining: &Pubkey,
    user_token_account: &Pubkey,
    user: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*reward_pool_spl, false),
        AccountMeta::new_readonly(*liquidity_mint, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new(*user_token_account, false),
        AccountMeta::new(*user, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::DepositMining { amount },
        accounts,
    )
}

/// Creates 'WithdrawMining' instruction.
#[allow(clippy::too_many_arguments)]
pub fn withdraw_mining(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_pool_spl: &Pubkey,
    reward_pool_authority: &Pubkey,
    liquidity_mint: &Pubkey,
    mining: &Pubkey,
    user_token_account: &Pubkey,
    user: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new(*reward_pool_spl, false),
        AccountMeta::new_readonly(*reward_pool_authority, false),
        AccountMeta::new_readonly(*liquidity_mint, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new(*user_token_account, false),
        AccountMeta::new(*user, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(clock::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::WithdrawMining, accounts)
}

/// Creates 'UpgradeMining' instruction.
#[allow(clippy::too_many_arguments)]
pub fn upgrade_mining(
    program_id: &Pubkey,
    rewards_root: &Pubkey,
    reward_pool: &Pubkey,
    mining: &Pubkey,
    user: &Pubkey,
    authority: &Pubkey,
    tier: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*rewards_root, false),
        AccountMeta::new_readonly(*reward_pool, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new(*user, false),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(clock::id(), false),
    ];

    Instruction::new_with_borsh(
        *program_id,
        &RewardsInstruction::UpgradeMining { tier },
        accounts,
    )
}

/// Creates 'Claim' instruction.
#[allow(clippy::too_many_arguments)]
pub fn claim(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
    vault: &Pubkey,
    mining: &Pubkey,
    user: &Pubkey,
    user_reward_token: &Pubkey,
) -> Instruction {
    println!("reward_pool: {}", reward_pool);
    println!("reward_mint: {}", reward_mint);
    println!("vault: {}", vault);
    println!("mining: {}", mining);
    println!("user: {}", user);
    println!("user_reward_token: {}", user_reward_token);

    let accounts = vec![
        AccountMeta::new_readonly(*reward_pool, false),
        AccountMeta::new_readonly(*reward_mint, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*mining, false),
        AccountMeta::new(*user, true),
        AccountMeta::new(*user_reward_token, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::Claim, accounts)
}

/// Creates 'InitializeRoot' instruction.
pub fn initialize_root(
    program_id: &Pubkey,
    rewards_root: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*rewards_root, true),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::InitializeRoot, accounts)
}

/// Creates 'MigratePool' instruction.
pub fn migrate_pool(
    program_id: &Pubkey,
    root_account: &Pubkey,
    reward_pool: &Pubkey,
    payer: &Pubkey,
    liquidity_mint: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*root_account, false),
        AccountMeta::new(*reward_pool, false),
        AccountMeta::new_readonly(*liquidity_mint, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(*program_id, &RewardsInstruction::MigratePool, accounts)
}
