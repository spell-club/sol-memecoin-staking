use crate::utils::*;
use everlend_rewards::state::{Mining, RewardPool};
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::borrow::Borrow;

use super::TestRewards;

#[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let test_reward_pool = TestRewards::new(&mut context).await;

    let pool_mint = Keypair::new();

    let (reward_pool_pubkey, _) = test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &pool_mint, 0)
        .await
        .unwrap();

    let reward_mint = Keypair::new();
    create_mint(&mut context, &reward_mint).await.unwrap();

    let reward_period = 3600;
    test_reward_pool
        .add_vault(
            &mut context,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            100,
            1,
            reward_period, // odd amount to match testing warp_to_epoch
        )
        .await;

    RewardPool::unpack(
        &get_account(&mut context, &reward_pool_pubkey)
            .await
            .data
            .borrow(),
    )
    .unwrap();

    let reward_amount = 1_000_000;
    let rewarder = test_reward_pool
        .create_token_holder(
            &mut context,
            &reward_mint.pubkey(),
            10_000_000_000,
            reward_amount,
        )
        .await;

    test_reward_pool
        .fill_vault(
            &mut context,
            &rewarder,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            reward_amount,
        )
        .await
        .unwrap();

    // deposit tokens
    let deposit_amount = 50_000;
    let token_holder = test_reward_pool
        .create_token_holder(
            &mut context,
            &pool_mint.pubkey(),
            10_000_000_000,
            deposit_amount,
        )
        .await;

    let mining_account = test_reward_pool
        .deposit_mining(
            &mut context,
            &pool_mint.pubkey(),
            &token_holder.token_account,
            &token_holder.owner,
            deposit_amount,
        )
        .await
        .unwrap();

    let reward_tier = 2;
    test_reward_pool
        .upgrade_mining(
            &mut context,
            &pool_mint.pubkey(),
            &token_holder.owner.pubkey(),
            reward_tier,
        )
        .await;

    let mining = Mining::unpack(
        &get_account(&mut context, &mining_account)
            .await
            .data
            .borrow(),
    )
    .unwrap();

    assert_eq!(mining.reward_tier, reward_tier);

    println!("{:?}", mining)
}
