use cosmwasm_std::{OverflowError, StdError};
use cw_controllers::AdminError;
use thiserror::Error;

use pandora_dapp_base::DappError;

#[derive(Error, Debug, PartialEq)]
pub enum SubscriptionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("{0}")]
    DappError(#[from] DappError),

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

    #[error("The contributor you wanted to remove is not registered.")]
    ContributorNotRegistered,

    #[error("You cant claim before your next payday on {0}")]
    WaitForNextPayday(u64),

    #[error("Your contribution compensation expired")]
    ContributionExpired,

    #[error("emissions for this OS are already claimed")]
    EmissionsAlreadyClaimed,

    #[error("only the factory can register new subscribers")]
    CallerNotFactory,

    #[error("you need to deposit at least {0} to reactivate this OS")]
    InsufficientPayment(u64),
}

impl From<semver::Error> for SubscriptionError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
