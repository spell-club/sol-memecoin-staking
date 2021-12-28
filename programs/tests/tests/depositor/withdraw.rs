#![cfg(feature = "test-bpf")]

use crate::utils::*;
use everlend_liquidity_oracle::state::{DistributionArray, LiquidityDistribution};
use everlend_utils::find_program_address;
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::signer::Signer;

async fn setup() -> (
    ProgramTestContext,
    TestSPLTokenLending,
    TestPythOracle,
    TestPoolMarket,
    TestPool,
    TestPoolBorrowAuthority,
    TestPoolMarket,
    TestPool,
    LiquidityProvider,
    TestDepositor,
) {
    let (mut context, money_market, pyth_oracle) = presetup().await;

    let payer_pubkey = context.payer.pubkey();

    // 0. Prepare lending
    let reserve = money_market.get_reserve_data(&mut context).await;
    println!("{:#?}", reserve);

    let account = get_account(&mut context, &money_market.market_pubkey).await;
    let lending_market =
        spl_token_lending::state::LendingMarket::unpack_from_slice(account.data.as_slice())
            .unwrap();

    let authority_signer_seeds = &[
        &money_market.market_pubkey.to_bytes()[..32],
        &[lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, &spl_token_lending::id()).unwrap();

    println!("{:#?}", lending_market_authority_pubkey);

    let collateral_mint = get_mint_data(&mut context, &reserve.collateral.mint_pubkey).await;
    println!("{:#?}", collateral_mint);

    // 1. Prepare general pool

    let general_pool_market = TestPoolMarket::new();
    general_pool_market.init(&mut context).await.unwrap();

    let general_pool = TestPool::new(&general_pool_market, None);
    general_pool
        .create(&mut context, &general_pool_market)
        .await
        .unwrap();

    // 1.1 Add liquidity to general pool

    let liquidity_provider = add_liquidity_provider(&mut context, &general_pool, 9999 * EXP)
        .await
        .unwrap();

    general_pool
        .deposit(
            &mut context,
            &general_pool_market,
            &liquidity_provider,
            100 * EXP,
        )
        .await
        .unwrap();

    // 2. Prepare money market pool

    let mm_pool_market = TestPoolMarket::new();
    mm_pool_market.init(&mut context).await.unwrap();

    let mm_pool = TestPool::new(&mm_pool_market, Some(reserve.collateral.mint_pubkey));
    mm_pool.create(&mut context, &mm_pool_market).await.unwrap();

    // 3. Prepare depositor

    // 3.1. Prepare liquidity oracle

    let test_liquidity_oracle = TestLiquidityOracle::new();
    test_liquidity_oracle.init(&mut context).await.unwrap();

    let mut distribution = DistributionArray::default();
    distribution[0] = LiquidityDistribution {
        money_market: spl_token_lending::id(),
        percent: 500_000_000u64, // 50%
    };

    let test_token_distribution =
        TestTokenDistribution::new(general_pool.token_mint_pubkey, distribution);

    test_token_distribution
        .init(&mut context, &test_liquidity_oracle, payer_pubkey)
        .await
        .unwrap();

    test_token_distribution
        .update(
            &mut context,
            &test_liquidity_oracle,
            payer_pubkey,
            distribution,
        )
        .await
        .unwrap();

    let test_depositor = TestDepositor::new();
    test_depositor
        .init(&mut context, &general_pool_market, &test_liquidity_oracle)
        .await
        .unwrap();

    // 3.2 Create transit account for liquidity token
    test_depositor
        .create_transit(&mut context, &general_pool.token_mint_pubkey)
        .await
        .unwrap();

    // 3.3 Create transit account for collateral token
    test_depositor
        .create_transit(&mut context, &mm_pool.token_mint_pubkey)
        .await
        .unwrap();

    // 3.4 Create transit account for mm pool collateral token
    test_depositor
        .create_transit(&mut context, &mm_pool.pool_mint.pubkey())
        .await
        .unwrap();

    // 4. Prepare borrow authority
    let (depositor_authority, _) = find_program_address(
        &everlend_depositor::id(),
        &test_depositor.depositor.pubkey(),
    );
    let general_pool_borrow_authority =
        TestPoolBorrowAuthority::new(&general_pool, depositor_authority);
    general_pool_borrow_authority
        .create(
            &mut context,
            &general_pool_market,
            &general_pool,
            SHARE_ALLOWED,
        )
        .await
        .unwrap();

    // 5. Start rebalancing
    test_depositor
        .start_rebalancing(
            &mut context,
            &general_pool_market,
            &general_pool,
            &test_liquidity_oracle,
        )
        .await
        .unwrap();

    // 6. Deposit

    // Rates should be refreshed
    context.warp_to_slot(3).unwrap();
    pyth_oracle.update(&mut context, 3).await;
    // money_market.refresh_reserve(&mut context, 3).await;

    test_depositor
        .deposit(
            &mut context,
            &general_pool_market,
            &general_pool,
            &mm_pool_market,
            &mm_pool,
            &money_market,
            // 50 * EXP,
        )
        .await
        .unwrap();

    // 6.1 Decrease distribution & restart rebalancing

    distribution[0].percent = 300_000_000u64; // Decrease to 30%
    test_token_distribution
        .update(
            &mut context,
            &test_liquidity_oracle,
            payer_pubkey,
            distribution,
        )
        .await
        .unwrap();

    test_depositor
        .start_rebalancing(
            &mut context,
            &general_pool_market,
            &general_pool,
            &test_liquidity_oracle,
        )
        .await
        .unwrap();

    (
        context,
        money_market,
        pyth_oracle,
        general_pool_market,
        general_pool,
        general_pool_borrow_authority,
        mm_pool_market,
        mm_pool,
        liquidity_provider,
        test_depositor,
    )
}

#[tokio::test]
async fn success() {
    let (
        mut context,
        money_market,
        pyth_oracle,
        general_pool_market,
        general_pool,
        _general_pool_borrow_authority,
        mm_pool_market,
        mm_pool,
        _liquidity_provider,
        test_depositor,
    ) = setup().await;

    let reserve = money_market.get_reserve_data(&mut context).await;
    let reserve_balance_before =
        get_token_balance(&mut context, &reserve.liquidity.supply_pubkey).await;

    context.warp_to_slot(5).unwrap();
    pyth_oracle.update(&mut context, 5).await;
    // money_market.refresh_reserve(&mut context, 5).await;

    test_depositor
        .withdraw(
            &mut context,
            &general_pool_market,
            &general_pool,
            &mm_pool_market,
            &mm_pool,
            &money_market,
            // 20 * EXP,
        )
        .await
        .unwrap();

    let rebalancing = test_depositor
        .get_rebalancing_data(&mut context, &general_pool.token_mint_pubkey)
        .await;

    assert!(rebalancing.is_completed());
    assert_eq!(
        get_token_balance(&mut context, &mm_pool.token_account.pubkey()).await,
        30 * EXP,
    );
    assert_eq!(
        get_token_balance(&mut context, &reserve.liquidity.supply_pubkey).await,
        reserve_balance_before - 20 * EXP,
    );
}
