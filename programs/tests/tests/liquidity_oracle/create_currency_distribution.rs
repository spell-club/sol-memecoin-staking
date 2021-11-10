use crate::utils::*;
use everlend_liquidity_oracle::{
    id, state::DistributionArray, state::LiquidityDistribution,
};
use solana_program::{
    clock::Slot, instruction::InstructionError, pubkey::Pubkey,
};
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::TransactionError};

const WARP_SLOT: Slot = 3;
const CURRENCY: &str = "SOL";

#[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let test_liquidity_oracle = TestLiquidityOracle::new();
    test_liquidity_oracle.init(&mut context).await.unwrap();

    context.warp_to_slot(WARP_SLOT).unwrap();

    let mut distribution = DistributionArray::default();
    distribution[0] = LiquidityDistribution {
        money_market: Pubkey::new_unique(),
        percent: 100 as f64,
    };

    let test_currency_distribution =
        TestCurrencyDistribution::new(CURRENCY.to_string(), distribution);
    let authority = context.payer.pubkey();

    test_currency_distribution
        .init(&mut context, &test_liquidity_oracle, authority)
        .await
        .unwrap();

    let result_distribution = test_currency_distribution
        .get_data(&mut context, &id(), &test_liquidity_oracle)
        .await;

    assert_eq!(distribution, result_distribution.distribution);
    assert_eq!(WARP_SLOT, result_distribution.slot);
}

#[tokio::test]
async fn fail_second_time_init() {
    let mut context = program_test().start_with_context().await;

    let test_liquidity_oracle = TestLiquidityOracle::new();
    test_liquidity_oracle.init(&mut context).await.unwrap();

    context.warp_to_slot(WARP_SLOT).unwrap();

    let mut distribution = DistributionArray::default();
    distribution[0] = LiquidityDistribution {
        money_market: Pubkey::new_unique(),
        percent: 100 as f64,
    };

    let test_currency_distribution =
        TestCurrencyDistribution::new(CURRENCY.to_string(), distribution);
    let authority = context.payer.pubkey();

    test_currency_distribution
        .init(&mut context, &test_liquidity_oracle, authority)
        .await
        .unwrap();

    context.warp_to_slot(WARP_SLOT + 2).unwrap();

    assert_eq!(
        test_currency_distribution
            .init(&mut context, &test_liquidity_oracle, authority)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}
