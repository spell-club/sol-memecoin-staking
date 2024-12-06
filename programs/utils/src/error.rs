//! Error types

use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum EverlendError {
    /// Input account owner
    #[error("Input account owner")]
    InvalidAccountOwner,

    /// Math operation overflow
    #[error("Math operation overflow")]
    MathOverflow,

    /// Amount cannot be zero
    #[error("Amount cannot be zero")]
    ZeroAmount,

    /// Not implemented
    #[error("Instruction not implemented")]
    NotImplemented,

    /// Invalid reward vault
    #[error("Invalid reward vault")]
    InvalidRewardVault,

    /// Invalid reward tier
    #[error("Invalid reward tier")]
    InvalidRewardTier,

    #[error("Lock time is still active")]
    LockTimeStillActive,

    #[error("Pool is full")]
    PoolIsFull,
}

impl PrintProgramError for EverlendError {
    fn print<E>(&self) {
        msg!("Error: {}", &self.to_string());
    }
}

impl From<EverlendError> for ProgramError {
    fn from(e: EverlendError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for EverlendError {
    fn type_of() -> &'static str {
        "EverlendError"
    }
}
