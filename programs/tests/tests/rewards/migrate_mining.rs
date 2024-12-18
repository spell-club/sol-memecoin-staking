use crate::utils::*;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{signature::Keypair};
use std::borrow::Borrow;

use solana_sdk::signature::Signer;
use everlend_rewards::state::{Mining, DeprecatedMining, DeprecatedRewardIndex};
use crate::rewards::TestRewards;

// #[tokio::test]

async fn success() {
    let initial_balance = 100000;

    let mut context = program_test().start_with_context().await;

    let test_reward_pool = TestRewards::new(&mut context).await;
    let liquidity_mint = Keypair::new();

    let (_, _) = test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &liquidity_mint, 0, 1)
        .await
        .unwrap();

    let token_holder = test_reward_pool
        .create_token_holder(
            &mut context,
            &liquidity_mint.pubkey(),
            10_000_000_000,
            initial_balance,
        )
        .await;

    let first_deposit_amount = 1250;

    let mining_pubkey = test_reward_pool
        .deposit_mining(
            &mut context,
            &liquidity_mint.pubkey(),
            &token_holder.token_account,
            &token_holder.owner,
            first_deposit_amount,
        )
        .await
        .unwrap();

    let mining_account_info = get_account(&mut context, &mining_pubkey).await;
    let init_mining = Mining::unpack(&mining_account_info.data.borrow()).unwrap();

    // Setup reward pool account as pre-migration version
    let mining_old = DeprecatedMining {
        account_type: init_mining.account_type,
        reward_pool: init_mining.reward_pool,
        bump: init_mining.bump,
        amount: init_mining.amount,
        rewards_calculated_at: init_mining.rewards_calculated_at,
        owner: init_mining.owner,
        last_deposit_time: init_mining.last_deposit_time,
        reward_tier: init_mining.reward_tier,
        indexes: init_mining.indexes.iter().map(|i| DeprecatedRewardIndex { reward_mint: i.reward_mint, rewards: i.rewards }).collect(),
    };

    let mut mining_account = get_account(&mut context, &mining_pubkey).await;
    mining_account.data = mining_account.data[0..DeprecatedMining::LEN].to_vec();
    mining_old.pack_into_slice(&mut mining_account.data);
    context.set_account(&mining_pubkey, &mining_account.into());

    // migrate
    test_reward_pool.migrate_mining(&mut context, &liquidity_mint, &mining_pubkey).await.unwrap();

    let mining_account_info = get_account(&mut context, &mining_pubkey).await;
    let mining = Mining::unpack(&mining_account_info.data.borrow()).unwrap();

    assert_eq!(mining.reward_pool, init_mining.reward_pool);
    assert_eq!(mining.bump, init_mining.bump);
    assert_eq!(mining.owner, init_mining.owner);
    assert_eq!(mining.amount, init_mining.amount);
}
