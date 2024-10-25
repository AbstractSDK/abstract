use abstract_app::sdk::AbstractSdkError;
use abstract_app::std::AbstractError;
use abstract_app::AppError;
use cosmwasm_std::{
    CheckedMultiplyFractionError, DecimalRangeExceeded, OverflowError, StdError, Uint128,
};
use cw_asset::{AssetError, AssetInfo};
use cw_controllers::AdminError;
use thiserror::Error;

use crate::handlers::execute::MAX_UNSUBS;

#[derive(Error, Debug, PartialEq)]
pub enum SubscriptionError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error(transparent)]
    Asset(#[from] AssetError),

    #[error(transparent)]
    AdminError(#[from] AdminError),

    #[error(transparent)]
    DecimalError(#[from] DecimalRangeExceeded),

    #[error(transparent)]
    AppError(#[from] AppError),

    #[error(transparent)]
    Overflow(#[from] OverflowError),

    #[error(transparent)]
    CheckedMultiplyFractionError(#[from] CheckedMultiplyFractionError),

    #[error("This contract does not implement the cw20 swap function")]
    NoSwapAvailable {},

    #[error("The provided token is not the payment token {0}")]
    WrongToken(AssetInfo),

    #[error("It's required to use cw20 send message to add pay with cw20 tokens")]
    NotUsingCW20Hook {},

    #[error("emissions for this OS are already claimed")]
    EmissionsAlreadyClaimed {},

    #[error("you need to deposit at least {0} {1} to (re)subscribe")]
    InsufficientPayment(Uint128, String),

    #[error("Subscriber emissions are not enabled")]
    SubscriberEmissionsNotEnabled {},

    #[error("Redundant unsubscribe call")]
    NoOneUnsubbed {},

    #[error("Can't unsubscribe more than {MAX_UNSUBS}")]
    TooManyUnsubs {},

    #[error("Income averaging period can't be zero")]
    ZeroAveragePeriod {},
}
