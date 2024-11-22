pub mod add_vault;
pub mod claim;
pub mod deposit_mining;
pub mod fill_vault;
pub mod initialize_pool;
pub mod withdraw_mining;

use crate::utils::{
    add_token_holder, create_mint, get_account, get_token_balance, transfer_sol, BanksClientResult,
    TokenHolder,
};
use anchor_lang::Key;
use everlend_rewards::{
    find_mining_program_address, find_reward_pool_program_address,
    find_reward_pool_spl_program_address, find_vault_program_address,
};
use everlend_utils::find_program_address;
use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

#[derive(Debug)]
pub struct TestRewards {
    pub rewards_root: Keypair,
    pub root_authority: Keypair,
}

impl TestRewards {
    pub async fn new(context: &mut ProgramTestContext) -> Self {
        let rewards_root = Keypair::new();
        let root_authority = Keypair::new();

        transfer_sol(context, &root_authority.pubkey(), 10_000_000)
            .await
            .unwrap();

        Self {
            // token_mint_pubkey: env.liquidity.pubkey(),
            rewards_root,
            root_authority,
        }
    }

    pub fn get_pool_addresses(&self, liquidity_mint: &Pubkey) -> (Pubkey, Pubkey) {
        let (reward_pool, _) = find_reward_pool_program_address(
            &everlend_rewards::id(),
            &self.rewards_root.pubkey(),
            liquidity_mint,
        );

        let (reward_pool_spl, _) = find_reward_pool_spl_program_address(
            &everlend_rewards::id(),
            &reward_pool,
            liquidity_mint,
        );

        (reward_pool, reward_pool_spl)
    }

    pub async fn create_mint_and_initialize_pool(
        &self,
        context: &mut ProgramTestContext,
        liquidity_mint: &Keypair,
        lock_time_sec: u64,
    ) -> BanksClientResult<(Pubkey, Pubkey)> {
        create_mint(context, liquidity_mint).await.unwrap();

        let (reward_pool, reward_pool_spl) = self.get_pool_addresses(&liquidity_mint.pubkey());
        let (reward_pool_authority, _) =
            find_program_address(&everlend_rewards::id(), &reward_pool);

        let tx = Transaction::new_signed_with_payer(
            &[
                everlend_rewards::instruction::initialize_root(
                    &everlend_rewards::id(),
                    &self.rewards_root.pubkey(),
                    &self.root_authority.pubkey(),
                ),
                everlend_rewards::instruction::initialize_pool(
                    &everlend_rewards::id(),
                    &self.rewards_root.pubkey(),
                    &reward_pool,
                    &reward_pool_spl,
                    &reward_pool_authority,
                    &liquidity_mint.pubkey(),
                    &self.root_authority.pubkey(),
                    lock_time_sec,
                ),
            ],
            Some(&self.root_authority.pubkey()),
            &[&self.root_authority, &self.rewards_root],
            context.last_blockhash,
        );

        let _ = context.banks_client.process_transaction(tx).await;

        Ok((reward_pool, reward_pool_spl))
    }

    pub async fn deposit_mining(
        &self,
        context: &mut ProgramTestContext,
        liquidity_mint: &Pubkey,
        user_token_account: &Pubkey,
        user: &Keypair,
        amount: u64,
    ) -> BanksClientResult<Pubkey> {
        let (reward_pool, reward_pool_spl) = self.get_pool_addresses(liquidity_mint);

        let (mining_account, _) =
            find_mining_program_address(&everlend_rewards::id(), &user.pubkey(), &reward_pool);

        let tx = Transaction::new_signed_with_payer(
            &[everlend_rewards::instruction::deposit_mining(
                &everlend_rewards::id(),
                &reward_pool,
                &reward_pool_spl,
                &liquidity_mint,
                &mining_account,
                user_token_account,
                &user.pubkey(),
                amount,
            )],
            None,
            &[user],
            context.last_blockhash,
        );

        let _ = context.banks_client.process_transaction(tx).await;

        Ok(mining_account)
    }

    pub async fn withdraw_mining(
        &self,
        context: &mut ProgramTestContext,
        liquidity_mint: &Pubkey,
        user_token_account: &Pubkey,
        user: &Keypair,
    ) -> BanksClientResult<()> {
        let (reward_pool, reward_pool_spl) = self.get_pool_addresses(liquidity_mint);

        let (mining_account, _) =
            find_mining_program_address(&everlend_rewards::id(), &user.pubkey(), &reward_pool);

        let (reward_pool_authority, _) =
            find_program_address(&everlend_rewards::id(), &reward_pool);

        let tx = Transaction::new_signed_with_payer(
            &[everlend_rewards::instruction::withdraw_mining(
                &everlend_rewards::id(),
                &reward_pool,
                &reward_pool_spl,
                &reward_pool_authority,
                &liquidity_mint,
                &mining_account,
                user_token_account,
                &user.pubkey(),
            )],
            None,
            &[user],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn add_vault(
        &self,
        context: &mut ProgramTestContext,
        liquidity_mint: &Pubkey,
        reward_mint: &Pubkey,
        ratio_base: u64,
        ratio_quote: u64,
        reward_period_sec: u32,
        distribution_starts_at: u64,
    ) -> Pubkey {
        let (reward_pool, _) = self.get_pool_addresses(liquidity_mint);

        let (vault_pubkey, _) =
            find_vault_program_address(&everlend_rewards::id(), &reward_pool, reward_mint);

        let tx = Transaction::new_signed_with_payer(
            &[everlend_rewards::instruction::add_vault(
                &everlend_rewards::id(),
                &self.rewards_root.pubkey(),
                &reward_pool,
                reward_mint,
                &vault_pubkey,
                &self.root_authority.pubkey(),
                ratio_base,
                ratio_quote,
                reward_period_sec,
                distribution_starts_at,
                0,
            )],
            Some(&self.root_authority.pubkey()),
            &[&self.root_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        vault_pubkey
    }

    pub async fn fill_vault(
        &self,
        context: &mut ProgramTestContext,
        from: &TokenHolder,
        liquidity_mint: &Pubkey,
        reward_mint: &Pubkey,
        amount: u64,
    ) -> BanksClientResult<()> {
        let (reward_pool, _) = self.get_pool_addresses(liquidity_mint);
        let (vault_pubkey, _) =
            find_vault_program_address(&everlend_rewards::id(), &reward_pool, reward_mint);

        let tx = Transaction::new_signed_with_payer(
            &[everlend_rewards::instruction::fill_vault(
                &everlend_rewards::id(),
                &reward_pool,
                reward_mint,
                &vault_pubkey,
                &from.token_account.key(),
                &from.owner.pubkey(),
                amount,
            )],
            None,
            &[&from.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn claim(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        liquidity_mint: &Pubkey,
        reward_mint: &Pubkey,
        user_reward_token_account: &Keypair,
    ) -> BanksClientResult<()> {
        let (reward_pool, _) = self.get_pool_addresses(liquidity_mint);

        let (mining_account, _) =
            find_mining_program_address(&everlend_rewards::id(), &user.pubkey(), &reward_pool);

        let (vault_pubkey, _) =
            find_vault_program_address(&everlend_rewards::id(), &reward_pool, reward_mint);

        let tx = Transaction::new_signed_with_payer(
            &[everlend_rewards::instruction::claim(
                &everlend_rewards::id(),
                &reward_pool,
                reward_mint,
                &vault_pubkey,
                &mining_account,
                &user.pubkey(),
                &user_reward_token_account.pubkey(),
            )],
            None,
            &[user, user_reward_token_account],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn create_token_holder(
        &self,
        context: &mut ProgramTestContext,
        liquidity_mint: &Pubkey,
        sol_balance: u64,
        token_balance: u64,
    ) -> TokenHolder {
        // give spl tokens
        let token_holder = add_token_holder(context, liquidity_mint, token_balance)
            .await
            .unwrap();

        // give sol
        transfer_sol(context, &token_holder.owner.pubkey(), sol_balance)
            .await
            .unwrap();

        let check_sol_balance = get_account(context, &token_holder.owner.pubkey())
            .await
            .lamports;
        assert_eq!(check_sol_balance, sol_balance);

        let check_token_balance = get_token_balance(context, &token_holder.token_account).await;
        assert_eq!(check_token_balance, token_balance);

        println!(
            "sol balance: {} | token balance {}",
            sol_balance, token_balance
        );

        token_holder
    }
}
