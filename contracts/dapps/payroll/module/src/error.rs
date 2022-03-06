use cosmwasm_std::{OverflowError, StdError};
use cw_controllers::AdminError;
use pandora_os::core::treasury::dapp_base::error::BaseDAppError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PaymentError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    BaseDAppError(#[from] BaseDAppError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("This contract does not implement the cw20 swap function")]
    NoSwapAvailable {},

    #[error("The provided token is not the base token")]
    WrongToken {},

    #[error("It's required to use cw20 send message to add pay with cw20 tokens")]
    NotUsingCW20Hook {},

    #[error("The provided fee is invalid")]
    InvalidFee {},

    #[error("The actual amount of tokens transfered is different from the claimed amount.")]
    InvalidAmount {},

    #[error("The contributor you wanted to remove is not registered.")]
    ContributorNotRegistered,

    #[error("You cant claim before your next payday on {0}")]
    WaitForNextPayday(u64),

    #[error("Your contribution compensation expired")]
    ContributionExpired,
}
