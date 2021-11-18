#![cfg(feature = "test-bpf")]

use crate::utils::*;
use everlend_depositor::{find_program_address, find_transit_program_address};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

async fn setup() -> (ProgramTestContext, TestDepositor) {
    let mut context = presetup().await.0;

    let test_depositor = TestDepositor::new(None);
    test_depositor.init(&mut context).await.unwrap();

    (context, test_depositor)
}

#[tokio::test]
async fn success() {
    let (mut context, test_depositor) = setup().await;

    let token_mint = Keypair::new();

    create_mint(
        &mut context,
        &token_mint,
        &test_depositor.rebalancer.pubkey(),
    )
    .await
    .unwrap();

    test_depositor
        .create_transit(&mut context, &token_mint.pubkey())
        .await
        .unwrap();

    let (transit_pubkey, _) = find_transit_program_address(
        &everlend_depositor::id(),
        &test_depositor.depositor.pubkey(),
        &token_mint.pubkey(),
    );

    let (depositor_authority, _) = find_program_address(
        &everlend_depositor::id(),
        &test_depositor.depositor.pubkey(),
    );

    let transit = get_token_account_data(&mut context, &transit_pubkey).await;

    assert_eq!(transit.mint, token_mint.pubkey());
    assert_eq!(transit.owner, depositor_authority);
}
