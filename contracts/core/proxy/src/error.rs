use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TreasuryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Admin(#[from] ::cw_controllers::AdminError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("DApp is already whitelisted")]
    AlreadyInList {},

    #[error("DApp not found in whitelist")]
    NotInList {},

    #[error("Sender is not whitelisted")]
    SenderNotWhitelisted {},

    #[error(
        "Treasury balance too low, {} requested but it only has {}",
        requested,
        balance
    )]
    Broke {
        balance: Uint128,
        requested: Uint128,
    },
}
impl From<semver::Error> for TreasuryError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
