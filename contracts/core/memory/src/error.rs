use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MemoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("You must provide exactly two assets when adding liquidity")]
    NotTwoAssets {},

    #[error("{} is not part of the provided pool", id)]
    NotInPool { id: String },
}
