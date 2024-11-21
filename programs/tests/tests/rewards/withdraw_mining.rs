use std::borrow::Borrow;

use crate::{rewards::TestRewards, utils::*};
use everlend_rewards::state::{Mining, RewardPool};
use solana_program_test::*;
use solana_sdk::{program_pack::Pack, signature::Keypair, signer::Signer};

#[tokio::test]
async fn success() {
    let initial_balance = 100000;

    let mut context = program_test().start_with_context().await;

    let test_reward_pool = TestRewards::new(&mut context).await;
    let liquidity_mint = Keypair::new();

    let (reward_pool, reward_pool_spl) = test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &liquidity_mint)
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

    println!("reward_pool: {}", reward_pool);
    println!("reward_pool_spl: {}", reward_pool_spl);
    println!("liquidity_mint: {}", liquidity_mint.pubkey());
    println!("user_token_account: {}", token_holder.token_account);

    let first_deposit_amount = 1250;

    let mining_account = test_reward_pool
        .deposit_mining(
            &mut context,
            &liquidity_mint.pubkey(),
            &token_holder.token_account,
            &token_holder.owner,
            first_deposit_amount,
        )
        .await
        .unwrap();

    let mining_account_info = get_account(&mut context, &mining_account).await;
    let mining = Mining::unpack(&mining_account_info.data.borrow()).unwrap();

    assert_eq!(mining.reward_pool, reward_pool);
    assert_eq!(mining.owner, token_holder.owner.pubkey());
    assert_eq!(mining.share, first_deposit_amount);

    let reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();

    assert_eq!(reward_pool_account.total_share, first_deposit_amount);

    let token_balance = get_token_balance(&mut context, &token_holder.token_account).await;
    assert_eq!(token_balance, initial_balance - first_deposit_amount);

    let pool_token_balance = get_token_balance(&mut context, &reward_pool_spl).await;
    assert_eq!(pool_token_balance, first_deposit_amount);

    test_reward_pool
        .withdraw_mining(
            &mut context,
            &liquidity_mint.pubkey(),
            &token_holder.token_account,
            &token_holder.owner,
        )
        .await
        .unwrap();

    let reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();

    assert_eq!(reward_pool_account.total_share, 0);

    let token_balance = get_token_balance(&mut context, &token_holder.token_account).await;
    assert_eq!(token_balance, initial_balance);

    let pool_token_balance = get_token_balance(&mut context, &reward_pool_spl).await;
    assert_eq!(pool_token_balance, 0);

    assert_eq!(
        context
            .banks_client
            .get_account(mining_account)
            .await
            .unwrap()
            .is_none(),
        true
    );

    context.warp_to_slot(3).unwrap();

    test_reward_pool
        .withdraw_mining(
            &mut context,
            &liquidity_mint.pubkey(),
            &token_holder.token_account,
            &token_holder.owner,
        )
        .await
        .unwrap_err();
}
