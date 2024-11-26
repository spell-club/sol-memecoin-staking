//! Program instructions

mod add_vault;
mod claim;
mod deposit_mining;
mod fill_vault;
mod initialize_pool;
mod initialize_root;
mod migrate_pool;
mod update_vault;
mod upgrade_mining;
mod withdraw_mining;

pub use add_vault::*;
pub use claim::*;
pub use deposit_mining::*;
pub use fill_vault::*;
pub use initialize_pool::*;
pub use initialize_root::*;
pub use migrate_pool::*;
pub use update_vault::*;
pub use upgrade_mining::*;
pub use withdraw_mining::*;
