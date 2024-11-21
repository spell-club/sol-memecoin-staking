use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
};

/// Create account
#[allow(clippy::too_many_arguments)]
pub fn create_account<'a, S: Pack>(
    program_id: &Pubkey,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    signers_seeds: &[&[&[u8]]],
    rent: &Rent,
) -> ProgramResult {
    let ix = system_instruction::create_account(
        from.key,
        to.key,
        rent.minimum_balance(S::LEN),
        S::LEN as u64,
        program_id,
    );

    invoke_signed(&ix, &[from, to], signers_seeds)
}

/// Transfer
#[allow(clippy::too_many_arguments)]
pub fn transfer<'a>(
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    lamports: u64,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = system_instruction::transfer(from.key, to.key, lamports);

    invoke_signed(&ix, &[from, to], signers_seeds)
}

pub fn realloc_with_rent<'a, 'b>(
    acc: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    rent: &Rent,
    new_len: usize,
) -> ProgramResult {
    let balance = acc.lamports();
    let min_balance = rent.minimum_balance(new_len);

    // Send some lamports
    if balance.lt(&min_balance) {
        invoke(
            &system_instruction::transfer(payer.key, acc.key, min_balance - balance),
            &[payer.clone(), acc.clone()],
        )?;
    }

    // Realloc
    acc.realloc(new_len, false)
}

pub fn close_account<'a, 'b>(
    source_account_info: &'a AccountInfo<'b>,
    dest_account_info: &'a AccountInfo<'b>,
) -> ProgramResult {
    let dest_starting_lamports = dest_account_info.lamports();
    **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(source_account_info.lamports())
        .unwrap();
    **source_account_info.lamports.borrow_mut() = 0;

    source_account_info.assign(&system_program::ID);
    source_account_info.realloc(0, false).map_err(Into::into)
}
