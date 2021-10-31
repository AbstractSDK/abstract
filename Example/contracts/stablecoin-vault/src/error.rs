use thiserror::Error;

use cosmwasm_std::StdError;
use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum StableVaultError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Call is not a callback!")]
    NotCallback {},

    #[error("No swaps can be performed in this pool")]
    NoSwapAvailable {},

    #[error("Initialization values make no sense.")]
    InvalidInit {},

    #[error("Not enough funds to perform trade")]
    Broke {},

    #[error("The requesting contract is not whitelisted.")]
    NotWhitelisted {},

    #[error("The requesting contract already whitelisted.")]
    AlreadyWhitelisted {},
}
