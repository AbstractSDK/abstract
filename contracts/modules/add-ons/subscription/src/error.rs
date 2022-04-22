use cosmwasm_std::{OverflowError, StdError};
use cw_controllers::AdminError;
use pandora_os::modules::dapp_base::error::BaseDAppError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SubscriptionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("{0}")]
    BaseDAppError(#[from] BaseDAppError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("This contract does not implement the cw20 swap function")]
    NoSwapAvailable {},

    #[error("The provided token is not a payment token")]
    WrongToken {},

    #[error("It's required to use cw20 send message to add pay with cw20 tokens")]
    NotUsingCW20Hook {},

    #[error("The provided fee is invalid")]
    InvalidFee {},

    #[error("The actual amount of tokens transferred is different from the claimed amount.")]
    InvalidAmount {},

    #[error("The provided native coin is not the same as the claimed deposit")]
    WrongNative {},

}

impl From<semver::Error> for SubscriptionError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
