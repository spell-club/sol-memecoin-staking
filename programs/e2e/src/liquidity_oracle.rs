use crate::utils::*;
use everlend_liquidity_oracle::{
    find_liquidity_oracle_token_distribution_program_address, instruction,
    state::{DistributionArray, LiquidityOracle},
};
use solana_client::client_error::ClientError;
use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

pub fn init(config: &Config, oracle_keypair: Option<Keypair>) -> Result<Pubkey, ClientError> {
    let oracle_keypair = oracle_keypair.unwrap_or_else(Keypair::new);

    println!("Liquidity oracle: {}", oracle_keypair.pubkey());

    let balance = config
        .rpc_client
        .get_minimum_balance_for_rent_exemption(LiquidityOracle::LEN)?;

    let tx = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &config.fee_payer.pubkey(),
                &oracle_keypair.pubkey(),
                balance,
                LiquidityOracle::LEN as u64,
                &everlend_liquidity_oracle::id(),
            ),
            instruction::init_liquidity_oracle(
                &everlend_liquidity_oracle::id(),
                &oracle_keypair.pubkey(),
                &config.fee_payer.pubkey(),
            ),
        ],
        Some(&config.fee_payer.pubkey()),
    );

    sign_and_send_and_confirm_transaction(config, tx, &[&config.fee_payer, &oracle_keypair])?;

    Ok(oracle_keypair.pubkey())
}

pub fn create_token_distribution(
    config: &Config,
    oracle_pubkey: &Pubkey,
    token_mint: &Pubkey,
    distribution: &DistributionArray,
) -> Result<Pubkey, ClientError> {
    let tx = Transaction::new_with_payer(
        &[instruction::create_token_distribution(
            &everlend_liquidity_oracle::id(),
            oracle_pubkey,
            &config.fee_payer.pubkey(),
            token_mint,
            *distribution,
        )],
        Some(&config.fee_payer.pubkey()),
    );

    sign_and_send_and_confirm_transaction(config, tx, &[&config.fee_payer])?;

    let (token_distribution_pubkey, _) = find_liquidity_oracle_token_distribution_program_address(
        &everlend_liquidity_oracle::id(),
        oracle_pubkey,
        token_mint,
    );

    Ok(token_distribution_pubkey)
}

pub fn update_token_distribution(
    config: &Config,
    oracle_pubkey: &Pubkey,
    token_mint: &Pubkey,
    distribution: &DistributionArray,
) -> Result<Pubkey, ClientError> {
    let tx = Transaction::new_with_payer(
        &[instruction::update_token_distribution(
            &everlend_liquidity_oracle::id(),
            oracle_pubkey,
            &config.fee_payer.pubkey(),
            token_mint,
            *distribution,
        )],
        Some(&config.fee_payer.pubkey()),
    );

    sign_and_send_and_confirm_transaction(config, tx, &[&config.fee_payer])?;

    let (token_distribution_pubkey, _) = find_liquidity_oracle_token_distribution_program_address(
        &everlend_liquidity_oracle::id(),
        oracle_pubkey,
        token_mint,
    );

    Ok(token_distribution_pubkey)
}