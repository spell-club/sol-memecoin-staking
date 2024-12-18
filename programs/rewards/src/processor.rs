//! Program processor

use crate::instruction::RewardsInstruction;
use crate::instructions::*;
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::pubkey::Pubkey;
use everlend_utils::EverlendError;

///
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = RewardsInstruction::try_from_slice(input)?;

    match instruction {
        RewardsInstruction::InitializePool {
            lock_time_sec,
            max_stakers,
        } => {
            msg!("RewardsInstruction: InitializePool");
            InitializePoolContext::new(program_id, accounts)?.process(
                program_id,
                lock_time_sec,
                max_stakers,
            )
        }
        RewardsInstruction::AddVault {
            reward_period_sec,
            is_enabled,
            tiers,
        } => {
            msg!("RewardsInstruction: AddVault");
            AddVaultContext::new(program_id, accounts)?.process(
                program_id,
                reward_period_sec,
                is_enabled,
                tiers,
            )
        }
        RewardsInstruction::UpdateVault {
            reward_period_sec,
            is_enabled,
            tiers,
        } => {
            msg!("RewardsInstruction: AddVault");
            UpdateVaultContext::new(program_id, accounts)?.process(
                program_id,
                reward_period_sec,
                is_enabled,
                tiers,
            )
        }

        RewardsInstruction::FillVault { amount } => {
            msg!("RewardsInstruction: FillVault");
            FillVaultContext::new(program_id, accounts)?.process(program_id, amount)
        }
        RewardsInstruction::DepositMining { amount } => {
            msg!("RewardsInstruction: DepositMining");
            DepositMiningContext::new(program_id, accounts)?.process(program_id, amount)
        }
        RewardsInstruction::WithdrawMining => {
            msg!("RewardsInstruction: WithdrawMining");
            WithdrawMiningContext::new(program_id, accounts)?.process(program_id)
        }
        RewardsInstruction::Claim => {
            msg!("RewardsInstruction: Claim");
            ClaimContext::new(program_id, accounts)?.process(program_id)
        }
        RewardsInstruction::UpgradeMining { tier } => {
            msg!("RewardsInstruction: UpgradeMining");
            UpgradeMiningContext::new(program_id, accounts)?.process(program_id, tier)
        }
        RewardsInstruction::InitializeRoot => {
            msg!("RewardsInstruction: InitializeRoot");
            InitializeRootContext::new(program_id, accounts)?.process(program_id)
        }
        RewardsInstruction::MigratePool { max_stakers, total_stakers } => {
            msg!("RewardsInstruction: MigratePool");
            Err(EverlendError::NotImplemented.into())
            // MigratePoolContext::new(program_id, accounts)?.process(program_id, max_stakers, total_stakers)
        }
        RewardsInstruction::MigrateMining => {
            msg!("RewardsInstruction: MigrateMining");
            Err(EverlendError::NotImplemented.into())
            // MigrateMiningContext::new(program_id, accounts)?.process(program_id)
        }
    }
}
