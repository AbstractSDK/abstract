use abstract_app::AppError;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContributorsError {
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
    DappError(#[from] AppError),

    #[error("You must wait one TWA period before claiming can start")]
    AveragingPeriodNotPassed {},

    #[error("income target is zero, no contributions can be paid out.")]
    TargetIsZero {},

    #[error("contributor must be a manager address")]
    ContributorNotManager {},
    
    #[error("compensation does not yield you any assets.")]
    NoAssetsToSend {},

    #[error("You can't claim before the end of the current period.")]
    CompensationAlreadyClaimed {},
}
