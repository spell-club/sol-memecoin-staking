use super::{create_token_account, mint_tokens, BanksClientResult};
use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::signature::{Keypair, Signer};

pub trait User {
    fn pubkey(&self) -> Pubkey;
}

#[derive(Debug)]
pub struct LiquidityProvider {
    pub owner: Keypair,
    pub token_account: Pubkey,
    pub pool_account: Pubkey,
}

impl User for LiquidityProvider {
    fn pubkey(&self) -> Pubkey {
        self.owner.pubkey()
    }
}

#[derive(Debug)]
pub struct TokenHolder {
    pub owner: Keypair,
    pub token_account: Pubkey,
}

impl User for TokenHolder {
    fn pubkey(&self) -> Pubkey {
        self.owner.pubkey()
    }
}

pub async fn add_token_holder(
    context: &mut ProgramTestContext,
    token_mint_pubkey: &Pubkey,
    mint_amount: u64,
) -> BanksClientResult<TokenHolder> {
    let user = Keypair::new();
    let token_account = Keypair::new();

    create_token_account(
        context,
        &token_account,
        token_mint_pubkey,
        &user.pubkey(),
        0,
    )
    .await?;

    mint_tokens(
        context,
        token_mint_pubkey,
        &token_account.pubkey(),
        mint_amount,
    )
    .await?;

    Ok(TokenHolder {
        owner: user,
        token_account: token_account.pubkey(),
    })
}
