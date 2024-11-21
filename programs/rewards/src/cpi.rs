//! CPI

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;

/// Rewards deposit mining
#[allow(clippy::too_many_arguments)]
pub fn deposit_mining<'a>(
    program_id: &Pubkey,
    reward_pool: AccountInfo<'a>,
    reward_pool_spl: AccountInfo<'a>,
    liquidity_mint: AccountInfo<'a>,
    mining: AccountInfo<'a>,
    user_token_account: AccountInfo<'a>,
    user: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = crate::instruction::deposit_mining(
        program_id,
        reward_pool.key,
        reward_pool_spl.key,
        liquidity_mint.key,
        mining.key,
        user_token_account.key,
        user.key,
        amount,
    );

    invoke_signed(&ix, &[reward_pool, mining, user], signers_seeds)
}

/// Rewards withdraw mining
#[allow(clippy::too_many_arguments)]
pub fn withdraw_mining<'a>(
    program_id: &Pubkey,
    reward_pool: AccountInfo<'a>,
    reward_pool_spl: AccountInfo<'a>,
    reward_pool_authority: AccountInfo<'a>,
    liquidity_mint: AccountInfo<'a>,
    mining: AccountInfo<'a>,
    user_token_account: AccountInfo<'a>,
    user: AccountInfo<'a>,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = crate::instruction::withdraw_mining(
        program_id,
        reward_pool.key,
        reward_pool_spl.key,
        reward_pool_authority.key,
        liquidity_mint.key,
        mining.key,
        user_token_account.key,
        user.key,
    );

    invoke_signed(&ix, &[reward_pool, mining, user], signers_seeds)
}

/// Rewards fill vault
#[allow(clippy::too_many_arguments)]
pub fn fill_vault<'a>(
    program_id: &Pubkey,
    reward_pool: AccountInfo<'a>,
    reward_mint: AccountInfo<'a>,
    vault: AccountInfo<'a>,
    from: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = crate::instruction::fill_vault(
        program_id,
        reward_pool.key,
        reward_mint.key,
        vault.key,
        from.key,
        authority.key,
        amount,
    );

    invoke_signed(
        &ix,
        &[reward_pool, reward_mint, vault, from, authority],
        signers_seeds,
    )
}
