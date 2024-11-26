#![allow(dead_code)]

use solana_program::{ed25519_program, program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::*;
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::clock::Clock;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::sysvar::clock;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub mod users;
pub use users::*;

pub const EXP: u64 = 1_000_000_000;
pub const REFRESH_INCOME_INTERVAL: u64 = 300; // About 2.5 min

pub type BanksClientResult<T> = Result<T, BanksClientError>;

pub struct TestEnvironment {
    pub context: ProgramTestContext,
    pub liquidity: Keypair,
}

pub fn program_test() -> ProgramTest {
    let mut program = ProgramTest::new(
        "everlend_rewards",
        everlend_rewards::id(),
        processor!(everlend_rewards::processor::process_instruction),
    );

    program.add_program(
        "spl_token_lending",
        spl_token_lending::id(),
        processor!(spl_token_lending::processor::process_instruction),
    );

    program
}

pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn get_clock(context: &mut ProgramTestContext) -> (Clock, Account) {
    let clockid = clock::id();
    let acc = get_account(context, &clockid).await;
    let clock: Clock = bincode::deserialize(&acc.data).unwrap();
    (clock, acc)
}

pub async fn get_mint_data(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) -> spl_token::state::Mint {
    let account = get_account(context, pubkey).await;
    spl_token::state::Mint::unpack_from_slice(account.data.as_slice()).unwrap()
}

pub async fn get_token_account_data(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
) -> spl_token::state::Account {
    let account = get_account(context, pubkey).await;
    spl_token::state::Account::unpack_from_slice(account.data.as_slice()).unwrap()
}

pub async fn get_token_balance(context: &mut ProgramTestContext, pubkey: &Pubkey) -> u64 {
    let account_info = get_token_account_data(context, pubkey).await;
    account_info.amount
}

pub fn get_liquidity_mint() -> (Keypair, Pubkey) {
    let keypair = read_keypair_file("tests/fixtures/lending/liquidity.json").unwrap();
    let pubkey = keypair.pubkey();

    (keypair, pubkey)
}

pub async fn transfer_sol(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
    amount: u64,
) -> BanksClientResult<()> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            pubkey,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn token_transfer(
    context: &mut ProgramTestContext,
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Keypair,
    amount: u64,
) -> BanksClientResult<()> {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::transfer(
            &spl_token::id(),
            source,
            destination,
            &authority.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
    lamports: u64,
) -> BanksClientResult<()> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN) + lamports,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
) -> BanksClientResult<()> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &context.payer.pubkey(),
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
) -> BanksClientResult<()> {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &context.payer.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
