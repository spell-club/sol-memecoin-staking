use crate::utils::*;
use everlend_rewards::state::{Mining, RewardPool, RewardTier, RewardVault};
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::sysvar::clock;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::state::Account;
use std::borrow::Borrow;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec;

use super::TestRewards;

#[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let test_reward_pool = TestRewards::new(&mut context).await;

    let pool_mint = Keypair::new();

    let (reward_pool_pubkey, _) = test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &pool_mint, 0, 5)
        .await
        .unwrap();

    let reward_mint = Keypair::new();
    create_mint(&mut context, &reward_mint).await.unwrap();
    let (mut clock, mut clock_account) = get_clock(&mut context).await;

    let reward_period = 3600;
    let vault = test_reward_pool
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
    let exp_reward_amount = 500;
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

    // update solana clock
    clock.unix_timestamp += reward_period as i64;
    clock_account.data = bincode::serialize(&clock).unwrap();
    context.set_account(&clock::id(), &clock_account.into());
    context.warp_to_slot(10).unwrap();

    /////////////
    let user_reward_account = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward_account,
        &reward_mint.pubkey(),
        &token_holder.owner.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_reward_pool
        .claim(
            &mut context,
            &token_holder.owner,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            &user_reward_account.pubkey(),
        )
        .await
        .unwrap();

    let user_reward = Account::unpack(
        get_account(&mut context, &user_reward_account.pubkey())
            .await
            .data
            .borrow(),
    )
    .unwrap();

    assert_eq!(user_reward.amount, exp_reward_amount);

    let mining = Mining::unpack(
        &get_account(&mut context, &mining_account)
            .await
            .data
            .borrow(),
    )
    .unwrap();

    assert_eq!(mining.indexes[0].rewards, 0);
    assert_eq!(mining.indexes[0].claimed_total_rewards, exp_reward_amount);

    let vault_acc = Account::unpack(get_account(&mut context, &vault).await.data.borrow()).unwrap();
    assert_eq!(vault_acc.amount, reward_amount - exp_reward_amount);

    let vault_acc = RewardPool::unpack(
        get_account(&mut context, &reward_pool_pubkey)
            .await
            .data
            .borrow(),
    )
    .unwrap();
    assert_eq!(vault_acc.vaults[0].claimed_total_amount, exp_reward_amount);

    println!("vault: {:?}", vault_acc);
}

#[tokio::test]
async fn reward_calculation() {
    let base = 100_000_000;
    let quote = 1000;
    let period = 60_u32;

    check_mining_maths(base, quote, period, period, 0, 100_000_000, 1000);
    check_mining_maths(base, quote, period, period, 20, 100_000_000, 20);
    check_mining_maths(base, quote, period, period * 10, 20, 10_000_000_000, 200);

    check_mining_maths(base, quote, period, period / 2, 0, 100_000_000, 0);
    check_mining_maths(base, quote, period, period * 5, 0, 25_000_000, 1250);

    let base = 1_000;
    let quote = 1;

    check_mining_maths(base, quote, period, period, 0, 1_250, 1);
    check_mining_maths(base, quote, period, period * 2, 0, 1_250, 2);
    check_mining_maths(base, quote, period, period * 4, 0, 1_250, 5); // we round to the end of period
}

fn check_mining_maths(
    base: u64,
    quote: u64,
    period: u32,
    add_time: u32,
    max_amount: u64,
    deposit: u64,
    reward: u64,
) {
    let reward_pool = Keypair::new();
    let owner = Keypair::new();

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let vault = RewardVault {
        vault_token_account_bump: 0,
        reward_mint: Keypair::new().pubkey(),

        reward_period_sec: period,
        reward_tiers: vec![RewardTier {
            ratio_base: base,
            ratio_quote: quote,
            reward_max_amount_per_period: max_amount,
        }],
        is_enabled: true,
        enabled_at: current_timestamp,
        claimed_total_amount: 0,
    };

    let mut mining = Mining::initialize(reward_pool.pubkey(), 0, owner.pubkey());
    mining.amount = deposit;
    mining.rewards_calculated_at = current_timestamp;

    let new_timestamp = current_timestamp + add_time as u64;
    mining
        .refresh_rewards(vec![vault].iter(), new_timestamp)
        .unwrap();

    assert_eq!(mining.indexes[0].rewards, reward);
    assert_eq!(mining.rewards_calculated_at, new_timestamp);
}
