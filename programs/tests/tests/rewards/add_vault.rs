use crate::utils::*;
use everlend_rewards::state::{RewardPool, RewardTier};
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::{borrow::Borrow, vec};

use super::TestRewards;

#[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let test_reward_pool = TestRewards::new(&mut context).await;

    let pool_mint = Keypair::new();

    let (reward_pool, _) = test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &pool_mint, 0)
        .await
        .unwrap();

    let reward_mint = Keypair::new();
    create_mint(&mut context, &reward_mint).await.unwrap();

    let (clock, _) = get_clock(&mut context).await;

    test_reward_pool
        .add_vault(
            &mut context,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            125,
            36,
            60,
        )
        .await;

    let reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();
    let vault = reward_pool_account.vaults.get(0).unwrap();

    assert_eq!(vault.reward_mint, reward_mint.pubkey());
    assert_eq!(vault.reward_tiers[0].ratio_base, 125);
    assert_eq!(vault.reward_tiers[0].ratio_quote, 36);
    assert_eq!(vault.reward_period_sec, 60);
    assert_eq!(vault.is_enabled, true);
    assert_eq!(vault.enabled_at, clock.unix_timestamp as u64);

    // disable vault
    test_reward_pool
        .update_vault(
            &mut context,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            None,
            Some(false),
            None,
        )
        .await;

    let reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();
    let vault = reward_pool_account.vaults.get(0).unwrap();

    assert_eq!(vault.reward_mint, reward_mint.pubkey());
    assert_eq!(vault.reward_tiers[0].ratio_base, 125);
    assert_eq!(vault.reward_tiers[0].ratio_quote, 36);
    assert_eq!(vault.reward_period_sec, 60);
    assert_eq!(vault.is_enabled, false);
    assert_eq!(vault.enabled_at, clock.unix_timestamp as u64);

    // disable vault
    test_reward_pool
        .update_vault(
            &mut context,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            Some(120),
            Some(true),
            Some(vec![
                RewardTier {
                    ratio_base: 1,
                    ratio_quote: 2,
                    reward_max_amount_per_period: 3,
                },
                RewardTier {
                    ratio_base: 4,
                    ratio_quote: 5,
                    reward_max_amount_per_period: 6,
                },
            ]),
        )
        .await;

    let reward_pool_account =
        RewardPool::unpack(get_account(&mut context, &reward_pool).await.data.borrow()).unwrap();
    let vault = reward_pool_account.vaults.get(0).unwrap();

    assert_eq!(vault.reward_mint, reward_mint.pubkey());
    assert_eq!(vault.reward_tiers[0].ratio_base, 1);
    assert_eq!(vault.reward_tiers[0].ratio_quote, 2);
    assert_eq!(vault.reward_tiers[0].reward_max_amount_per_period, 3);
    assert_eq!(vault.reward_tiers[1].ratio_base, 4);
    assert_eq!(vault.reward_tiers[1].ratio_quote, 5);
    assert_eq!(vault.reward_tiers[1].reward_max_amount_per_period, 6);
    assert_eq!(vault.reward_period_sec, 120);
    assert_eq!(vault.is_enabled, true);
    assert_eq!(vault.enabled_at, clock.unix_timestamp as u64);
}
