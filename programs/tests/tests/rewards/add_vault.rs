use crate::utils::*;
use anchor_lang::Key;
use everlend_rewards::state::RewardPool;
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

    let (reward_pool, _) = test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &pool_mint)
        .await
        .unwrap();

    let reward_mint = Keypair::new();
    create_mint(&mut context, &reward_mint).await.unwrap();

    test_reward_pool
        .add_vault(&mut context, &pool_mint.pubkey(), &reward_mint.pubkey())
        .await;

    let reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();
    let vaults = reward_pool_account.vaults.get(0).unwrap();

    assert_eq!(vaults.reward_mint, reward_mint.pubkey());
}
