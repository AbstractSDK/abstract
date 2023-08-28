use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;

use thiserror::Error;

use abstract_app::AppError as AbstractAppError;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;

#[derive(Error, Debug, PartialEq)]
pub enum GasStationError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    DappError(#[from] AbstractAppError),

    #[error("Convert can be called only by the croncat manager")]
    NotManagerConvert {},

    #[error("Grade {0} already exists")]
    GradeAlreadyExists(String),

    #[error("Gas pass {0} already exists")]
    GasPassAlreadyExists(String),

    #[error("Grade {0} not found")]
    GradeNotFound(String),

    #[error("Holder {0} not found")]
    HolderNotFound(String),

    #[error("Denom {0} already exists")]
    DenomAlreadyExists(String),

    #[error("Pending gas pump {pending} does not match created gas pump {created}")]
    PendingGasPumpDoesNotMatchCreatedGasPump { pending: String, created: String },

    #[error("Only native tokens can be used as gas")]
    OnlyNativeTokensCanBeUsedAsGas {},
}
