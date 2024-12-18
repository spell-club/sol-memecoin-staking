use crate::state::{Mining, RewardPool, RewardsRoot, MAX_TIERS};
use everlend_utils::{assert_account_key, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::{clock, Sysvar};

/// Instruction context
pub struct UpgradeMiningContext<'a, 'b> {
    rewards_root: &'a AccountInfo<'b>,
    reward_pool: &'a AccountInfo<'b>,
    user: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    authority: &'a AccountInfo<'b>,
    clock: &'a AccountInfo<'b>,
}

impl<'a, 'b> UpgradeMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<UpgradeMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let rewards_root = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let user = AccountLoader::next_unchecked(account_info_iter)?;
        let authority = AccountLoader::next_signer(account_info_iter)?;
        let clock = AccountLoader::next_with_key(account_info_iter, &clock::id())?;

        Ok(UpgradeMiningContext {
            rewards_root,
            reward_pool,
            mining,
            user,
            authority,
            clock,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey, tier: u8) -> ProgramResult {
        let reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        assert_account_key(self.rewards_root, &reward_pool.rewards_root)?;

        {
            let rewards_root = RewardsRoot::unpack(&self.rewards_root.data.borrow())?;
            assert_account_key(self.authority, &rewards_root.authority)?;
        }

        let mut mining = Mining::unpack(&self.mining.data.borrow())?;
        {
            let mining_pubkey = Pubkey::create_program_address(
                &[
                    b"mining".as_ref(),
                    self.user.key.as_ref(),
                    self.reward_pool.key.as_ref(),
                    &[mining.bump],
                ],
                program_id,
            )?;
            assert_account_key(self.mining, &mining_pubkey)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            assert_account_key(self.user, &mining.owner)?;
        }

        let timestamp = Clock::from_account_info(self.clock)?.unix_timestamp;

        if tier > MAX_TIERS as u8 - 1 {
            return Err(ProgramError::InvalidArgument);
        }

        if tier == mining.reward_tier {
            return Err(ProgramError::InvalidArgument);
        }

        mining.refresh_rewards(reward_pool.vaults.iter(), timestamp as u64)?;
        mining.reward_tier = tier;

        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        Ok(())
    }
}
