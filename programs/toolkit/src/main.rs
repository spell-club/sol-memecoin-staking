use std::str::FromStr;

use dotenv::dotenv;
use everlend_rewards::{
    find_reward_pool_program_address, find_reward_pool_spl_token_account,
    find_vault_spl_token_account,
    instruction::{add_vault, fill_vault, initialize_pool, initialize_root, update_vault},
    state::RewardTier,
};
use solana_client::rpc_client::RpcClient;
use solana_program::hash::Hash;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

const DEVNET_EVERLEND_USDC_MINT: &str = "5gAhHuS82QgoYrkREuWUgVVqUYLdc8SsuiYtHESVvHcZ";

fn main() {
    dotenv().ok();

    let reward_authority_seed = std::env::var("REWARD_AUTHORITY_SEED_BASE58")
        .expect("REWARD_AUTHORITY_SEED_BASE58 must be set.");
    let rewards_root_seed =
        std::env::var("REWARDS_ROOT_SEED_BASE58").expect("REWARDS_ROOT_SEED_BASE58 must be set.");

    // RPC connection to a Solana cluster (devnet in this example)
    let rpc_url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new(rpc_url);

    let reward_authority = Keypair::from_base58_string(&reward_authority_seed);
    let rewards_root = Keypair::from_base58_string(&rewards_root_seed);

    let liquidity_mint = Pubkey::from_str(DEVNET_EVERLEND_USDC_MINT).unwrap();

    // send_initialize_pool(
    //     &rpc_client,
    //     &reward_authority,
    //     &rewards_root.pubkey(),
    //     &liquidity_mint,
    // );

    let reward_mint = Pubkey::from_str("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr").unwrap();
    // send_add_vault(
    //     &rpc_client,
    //     &reward_authority,
    //     &rewards_root.pubkey(),
    //     &liquidity_mint,
    //     &reward_mint,
    // );

    send_update_vault(
        &rpc_client,
        &reward_authority,
        &rewards_root.pubkey(),
        &liquidity_mint,
        &reward_mint,
    );

    // send_fill_vault(
    //     &rpc_client,
    //     &reward_authority,
    //     &rewards_root.pubkey(),
    //     &liquidity_mint,
    //     &reward_mint,
    // )
}

fn send_initialize_pool(
    rpc_client: &RpcClient,
    reward_authority: &Keypair,
    rewards_root: &Pubkey,
    liquidity_mint: &Pubkey,
) {
    // Create the transaction instruction
    let (reward_pool, _) =
        find_reward_pool_program_address(&everlend_rewards::id(), rewards_root, liquidity_mint);

    let (reward_pool_spl, _) =
        find_reward_pool_spl_token_account(&everlend_rewards::id(), &reward_pool, liquidity_mint);

    let (reward_pool_authority, _) =
        Pubkey::find_program_address(&[&reward_pool.to_bytes()[..32]], &everlend_rewards::id());

    let ix = initialize_pool(
        &everlend_rewards::id(),
        &rewards_root,
        &reward_pool,
        &reward_pool_spl,
        &reward_pool_authority,
        liquidity_mint,
        &reward_authority.pubkey(),
        0,
        1000,
    );

    // Build the transaction
    let transaction = Transaction::new_signed_with_payer(
        &[ix], // Instructions to execute
        Some(&reward_authority.pubkey()),
        &[&reward_authority],       // Signers
        get_blockhash(&rpc_client), // Recent blockhash
    );

    send_tx(&rpc_client, transaction);
}

fn send_initialize_root(
    rpc_client: &RpcClient,
    rewards_root: &Keypair,
    reward_authority: &Keypair,
) {
    // Create the transaction instruction
    let ix = initialize_root(
        &everlend_rewards::id(),
        &rewards_root.pubkey(),
        &reward_authority.pubkey(),
    );

    // Build the transaction
    let transaction = Transaction::new_signed_with_payer(
        &[ix], // Instructions to execute
        Some(&reward_authority.pubkey()),
        &[&reward_authority, &rewards_root], // Signers
        get_blockhash(&rpc_client),          // Recent blockhash
    );

    send_tx(&rpc_client, transaction);
}

fn send_add_vault(
    rpc_client: &RpcClient,
    reward_authority: &Keypair,
    rewards_root: &Pubkey,
    liquidity_mint: &Pubkey,
    reward_mint: &Pubkey,
) {
    // Create the transaction instruction
    let (reward_pool, _) =
        find_reward_pool_program_address(&everlend_rewards::id(), rewards_root, liquidity_mint);

    let (vault, _) =
        find_vault_spl_token_account(&everlend_rewards::id(), &reward_pool, &reward_mint);

    let ix = add_vault(
        &everlend_rewards::id(),
        &rewards_root,
        &reward_pool,
        &reward_mint,
        &vault,
        &reward_authority.pubkey(),
        60,
        vec![
            RewardTier {
                ratio_base: 1000,
                ratio_quote: 1,
                reward_max_amount_per_period: 2,
            },
            RewardTier {
                ratio_base: 1000,
                ratio_quote: 2,
                reward_max_amount_per_period: 4,
            },
        ],
    );

    // Build the transaction
    let transaction = Transaction::new_signed_with_payer(
        &[ix], // Instructions to execute
        Some(&reward_authority.pubkey()),
        &[&reward_authority],       // Signers
        get_blockhash(&rpc_client), // Recent blockhash
    );

    send_tx(&rpc_client, transaction);
}

fn send_update_vault(
    rpc_client: &RpcClient,
    reward_authority: &Keypair,
    rewards_root: &Pubkey,
    liquidity_mint: &Pubkey,
    reward_mint: &Pubkey,
) {
    // Create the transaction instruction
    let (reward_pool, _) =
        find_reward_pool_program_address(&everlend_rewards::id(), rewards_root, liquidity_mint);

    let ix = update_vault(
        &everlend_rewards::id(),
        &rewards_root,
        &reward_pool,
        &reward_mint,
        &reward_authority.pubkey(),
        None,
        None,
        Some(vec![
            RewardTier {
                ratio_base: 1000,
                ratio_quote: 1,
                reward_max_amount_per_period: 2,
            },
            RewardTier {
                ratio_base: 1000,
                ratio_quote: 2,
                reward_max_amount_per_period: 4,
            },
            RewardTier {
                ratio_base: 1000,
                ratio_quote: 4,
                reward_max_amount_per_period: 6,
            },
        ]),
    );

    // Build the transaction
    let transaction = Transaction::new_signed_with_payer(
        &[ix], // Instructions to execute
        Some(&reward_authority.pubkey()),
        &[&reward_authority],       // Signers
        get_blockhash(&rpc_client), // Recent blockhash
    );

    send_tx(&rpc_client, transaction);
}

fn send_fill_vault(
    rpc_client: &RpcClient,
    reward_authority: &Keypair,
    rewards_root: &Pubkey,
    liquidity_mint: &Pubkey,
    reward_mint: &Pubkey,
) {
    // Create the transaction instruction
    let (reward_pool, _) =
        find_reward_pool_program_address(&everlend_rewards::id(), rewards_root, liquidity_mint);

    let (vault, _) =
        find_vault_spl_token_account(&everlend_rewards::id(), &reward_pool, &reward_mint);

    let (token_account, _) = everlend_utils::cpi::spl_token::find_associated_token_account(
        &reward_authority.pubkey(),
        reward_mint,
    );

    let ix = fill_vault(
        &everlend_rewards::id(),
        &reward_pool,
        &reward_mint,
        &vault,
        &token_account,
        &reward_authority.pubkey(),
        1000,
    );

    // Build the transaction
    let transaction = Transaction::new_signed_with_payer(
        &[ix], // Instructions to execute
        Some(&reward_authority.pubkey()),
        &[&reward_authority],       // Signers
        get_blockhash(&rpc_client), // Recent blockhash
    );

    send_tx(&rpc_client, transaction);
}

fn get_blockhash(rpc_client: &RpcClient) -> Hash {
    // Get the latest blockhash
    rpc_client
        .get_latest_blockhash()
        .expect("Failed to get blockhash")
}
fn send_tx(rpc_client: &RpcClient, transaction: Transaction) {
    // Send the transaction
    let signature = rpc_client
        .send_and_confirm_transaction(&transaction)
        .expect("Failed to send transaction");

    println!("Transaction successful! Signature: {}", signature);
}
