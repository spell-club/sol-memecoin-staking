use crate::utils::*;
use everlend_liquidity_oracle::{
    id, state::DistributionArray, state::LiquidityDistribution,
};
use solana_program::{
    clock::Slot, pubkey::Pubkey,
};
use solana_program_test::*;
use solana_sdk::{signer::Signer};

const CURRENCY: &str = "SOL";
const WARP_SLOT: Slot = 3;

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

    context.warp_to_slot(WARP_SLOT + 2).unwrap();

    distribution[0] = LiquidityDistribution {
        money_market: Pubkey::new_unique(),
        percent: 90 as f64,
    };
    distribution[1] = LiquidityDistribution {
        money_market: Pubkey::new_unique(),
        percent: 10 as f64,
    };

    test_currency_distribution
        .update(
            &mut context,
            &test_liquidity_oracle,
            authority,
            distribution,
        )
        .await
        .unwrap();

    context.warp_to_slot(WARP_SLOT + 4).unwrap();

    let result_distribution = test_currency_distribution
        .get_data(&mut context, &id(), &test_liquidity_oracle)
        .await;

    assert_eq!(distribution, result_distribution.distribution);
    assert_eq!(WARP_SLOT + 2, result_distribution.slot);
}
