use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Admin(#[from] ::cw_controllers::AdminError),

    #[error(transparent)]
    SemVer(#[from] ::semver::Error),

    #[error("DApp is already whitelisted")]
    AlreadyInList {},

    #[error("DApp not found in whitelist")]
    NotInList {},

    #[error("Sender is not whitelisted")]
    SenderNotWhitelisted {},

    #[error("Max amount of assets registered")]
    AssetsLimitReached,

    #[error("Max amount of modules registered")]
    ModuleLimitReached,

    #[error("The proposed update resulted in a bad configuration: {0}")]
    BadUpdate(String),

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
