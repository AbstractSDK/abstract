use abstract_app::AppError;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::{CheckedMultiplyFractionError, DecimalRangeExceeded, OverflowError, StdError};
use cw_asset::{AssetError, AssetInfo};
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SubscriptionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    AdminError(#[from] AdminError),

    #[error("{0}")]
    DecimalError(#[from] DecimalRangeExceeded),

    #[error("{0}")]
    AppError(#[from] AppError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    CheckedMultiplyFractionError(#[from] CheckedMultiplyFractionError),

    #[error("This contract does not implement the cw20 swap function")]
    NoSwapAvailable {},

    #[error("The provided token is not the payment token {0}")]
    WrongToken(AssetInfo),

    #[error("It's required to use cw20 send message to add pay with cw20 tokens")]
    NotUsingCW20Hook {},

    #[error("emissions for this OS are already claimed")]
    EmissionsAlreadyClaimed {},

    #[error("you need to deposit at least {0} {1} to (re)activate this OS")]
    InsufficientPayment(u64, String),

    #[error("Subscriber emissions are not enabled")]
    SubscriberEmissionsNotEnabled {},

    #[error("Redundant unsubscribe call")]
    NoOneUnsubbed {},
}
