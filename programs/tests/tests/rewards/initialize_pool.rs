use crate::utils::*;
use everlend_rewards::state::RewardPool;
use everlend_utils::find_program_address;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::state::Account;
use std::borrow::Borrow;

use super::TestRewards;

#[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let test_reward_pool = TestRewards::new(&mut context).await;

    let pool_mint = Keypair::new();
    let lock_time_sec = 60;
    let max_stakers = 5;

    let (reward_pool, reward_pool_spl) = test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &pool_mint, lock_time_sec, max_stakers)
        .await
        .unwrap();

    let reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();

    assert_eq!(
        reward_pool_account.rewards_root,
        test_reward_pool.rewards_root.pubkey()
    );

    assert_eq!(reward_pool_account.liquidity_mint, pool_mint.pubkey());
    assert_eq!(reward_pool_account.lock_time_sec, lock_time_sec);
    assert_eq!(reward_pool_account.max_stakers, max_stakers);

    let reward_pool_spl_account = get_account(&mut context, &reward_pool_spl).await;
    let reward_pool_spl = Account::unpack(reward_pool_spl_account.data.borrow()).unwrap();

    let (reward_pool_authority, _) = find_program_address(&everlend_rewards::id(), &reward_pool);

    assert_eq!(reward_pool_account.liquidity_mint, reward_pool_spl.mint);
    assert_eq!(reward_pool_authority, reward_pool_spl.owner);
    assert_eq!(0, reward_pool_spl.amount);
}
