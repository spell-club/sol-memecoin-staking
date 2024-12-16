use crate::utils::*;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{signature::Keypair};
use std::borrow::Borrow;

use solana_sdk::signature::Signer;
use everlend_rewards::state::{DeprecatedRewardPool, RewardPool, InitRewardPoolParams};
use crate::rewards::TestRewards;

// #[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let test_reward_pool = TestRewards::new(&mut context).await;

    let pool_mint = Keypair::new();
    let lock_time_sec = 60;
    let max_stakers = 5;

    let (reward_pool, _) = test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &pool_mint, lock_time_sec, max_stakers)
        .await
        .unwrap();
    let init_reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();

    // Setup reward pool account as pre-migration version
    let reward_pool_old = DeprecatedRewardPool::init(InitRewardPoolParams {
        rewards_root: init_reward_pool_account.rewards_root,
        bump: init_reward_pool_account.bump,
        liquidity_mint: init_reward_pool_account.liquidity_mint,
        lock_time_sec: init_reward_pool_account.lock_time_sec,
        max_stakers,
    });

    let mut reward_pool_account = get_account(&mut context, &reward_pool).await;
    reward_pool_account.data = reward_pool_account.data[0..DeprecatedRewardPool::LEN].to_vec();
    reward_pool_old.pack_into_slice(&mut reward_pool_account.data);
    context.set_account(&reward_pool, &reward_pool_account.into());

    let max_stakers = 10;
    let total_stakers = 5;

    // migrate
    test_reward_pool.migrate_pool(&mut context, &pool_mint, max_stakers, total_stakers).await.unwrap();

    let reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();

    assert_eq!(
        reward_pool_account.rewards_root,
        test_reward_pool.rewards_root.pubkey()
    );

    assert_eq!(reward_pool_account.liquidity_mint, pool_mint.pubkey());
    assert_eq!(reward_pool_account.bump, init_reward_pool_account.bump);
    assert_eq!(reward_pool_account.lock_time_sec, lock_time_sec);
    assert_eq!(reward_pool_account.max_stakers, max_stakers);
    assert_eq!(reward_pool_account.total_stakers, total_stakers);
}
