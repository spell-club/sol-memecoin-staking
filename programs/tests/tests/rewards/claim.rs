use crate::utils::*;
use anchor_lang::Key;
use everlend_rewards::find_mining_program_address;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_token::state::Account;
use std::borrow::Borrow;

use super::TestRewards;

#[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let test_reward_pool = TestRewards::new(&mut context).await;

    let pool_mint = Keypair::new();

    test_reward_pool
        .create_mint_and_initialize_pool(&mut context, &pool_mint)
        .await
        .unwrap();

    let reward_mint = Keypair::new();
    create_mint(&mut context, &reward_mint).await.unwrap();

    let vault = test_reward_pool
        .add_vault(&mut context, &pool_mint.pubkey(), &reward_mint.pubkey())
        .await;

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

    /////////////
    let user_reward_account = Keypair::new();

    test_reward_pool
        .claim(
            &mut context,
            &token_holder.owner,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            &user_reward_account,
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

    assert_eq!(user_reward.amount, 1_000_000);
}

// #[tokio::test]
// async fn with_two_users() {
//     let (mut context, test_rewards, user1, fee, rewarder) = setup().await;

//     let user2 = Keypair::new();

//     test_rewards
//         .deposit_mining(&mut context, &user2, 50)
//         .await
//         .unwrap();

//     test_rewards
//         .fill_vault(&mut context, &fee, &rewarder, 1_000_000)
//         .await
//         .unwrap();

//     let user_reward1 = Keypair::new();
//     create_token_account(
//         &mut context,
//         &user_reward1,
//         &test_rewards.token_mint_pubkey,
//         &user1.pubkey(),
//         0,
//     )
//     .await
//     .unwrap();

//     test_rewards
//         .claim(&mut context, &user1, &user_reward1.pubkey())
//         .await
//         .unwrap();

//     let user_reward2 = Keypair::new();
//     create_token_account(
//         &mut context,
//         &user_reward2,
//         &test_rewards.token_mint_pubkey,
//         &user2.pubkey(),
//         0,
//     )
//     .await
//     .unwrap();

//     test_rewards
//         .claim(&mut context, &user2, &user_reward2.pubkey())
//         .await
//         .unwrap();

//     let user_reward_account1 = get_account(&mut context, &user_reward1.pubkey()).await;
//     let user_reward1 = Account::unpack(user_reward_account1.data.borrow()).unwrap();

//     assert_eq!(user_reward1.amount, 653_333);

//     let user_reward_account2 = get_account(&mut context, &user_reward2.pubkey()).await;
//     let user_reward2 = Account::unpack(user_reward_account2.data.borrow()).unwrap();

//     assert_eq!(user_reward2.amount, 326_666);
// }
