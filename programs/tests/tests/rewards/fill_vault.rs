use crate::utils::*;
use solana_program::program_pack::Pack;
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
        .create_mint_and_initialize_pool(&mut context, &pool_mint, 0, 5)
        .await
        .unwrap();

    let reward_mint = Keypair::new();
    create_mint(&mut context, &reward_mint).await.unwrap();

    let vault = test_reward_pool
        .add_vault(
            &mut context,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            1,
            1,
            60,
        )
        .await;

    let initial_balance = 1_000_000;
    let rewarder = test_reward_pool
        .create_token_holder(
            &mut context,
            &reward_mint.pubkey(),
            10_000_000_000,
            initial_balance,
        )
        .await;

    test_reward_pool
        .fill_vault(
            &mut context,
            &rewarder,
            &pool_mint.pubkey(),
            &reward_mint.pubkey(),
            initial_balance / 2,
        )
        .await
        .unwrap();

    let vault_account = get_account(&mut context, &vault).await;
    let rewarder_account = get_account(&mut context, &rewarder.token_account).await;

    let vault = Account::unpack(vault_account.data.borrow()).unwrap();
    let rewarder = Account::unpack(rewarder_account.data.borrow()).unwrap();

    assert_eq!(vault.amount, initial_balance / 2);
    assert_eq!(rewarder.amount, initial_balance / 2);
}
