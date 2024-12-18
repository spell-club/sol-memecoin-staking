use everlend_utils::cpi::system::realloc_with_rent;
use everlend_utils::{assert_account_key, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::{Sysvar, SysvarId};

use crate::state::{RewardPool, Mining, DeprecatedMining};

/// Instruction context
pub struct MigrateMiningContext<'a, 'b> {
    mining: &'a AccountInfo<'b>,
    rewards_root: &'a AccountInfo<'b>,
    reward_pool: &'a AccountInfo<'b>,
    liquidity_mint: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
}

impl<'a, 'b> MigrateMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<MigrateMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let rewards_root = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let liquidity_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let payer = AccountLoader::next_signer(account_info_iter)?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;
        let rent = AccountLoader::next_with_key(account_info_iter, &Rent::id())?;

        Ok(MigrateMiningContext {
            rewards_root,
            reward_pool,
            liquidity_mint,
            mining,
            payer,
            rent,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let rent = Rent::from_account_info(self.rent)?;

        let deprecated_mining = DeprecatedMining::unpack(&self.mining.data.borrow())?;
        let mining = Mining::migrate(&deprecated_mining);

        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;

        let reward_pool_seeds = &[
            b"reward_pool".as_ref(),
            &reward_pool.rewards_root.to_bytes()[..32],
            &reward_pool.liquidity_mint.to_bytes()[..32],
            &[reward_pool.bump],
        ];

        {
            assert_account_key(self.payer, &mining.owner)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            assert_account_key(
                self.reward_pool,
                &Pubkey::create_program_address(reward_pool_seeds, program_id)?,
            )?;
        }

        {
            let mining_pubkey = Pubkey::create_program_address(
                &[
                    b"mining".as_ref(),
                    self.payer.key.as_ref(),
                    self.reward_pool.key.as_ref(),
                    &[mining.bump],
                ],
                program_id,
            )?;
            assert_account_key(self.mining, &mining_pubkey)?;
        }

        realloc_with_rent(self.mining, self.payer, &rent, Mining::LEN)?;

        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        Ok(())
    }
}
