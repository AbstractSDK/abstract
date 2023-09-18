use abstract_app::AppError as AbstractAppError;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
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

    #[error("Resource not found")]
    NotFound {},

    #[error("Already checked in")]
    AlreadyCheckedIn {},

    #[error("Voter already voted")]
    AlreadyVoted {},

    #[error("Friend already vetoed")]
    AlreadyAdded {},

    #[error("Voter not found")]
    VoterNotFound {},

    #[error("The challenge status is not correct for this action")]
    WrongChallengeStatus {},

    #[error("The check in status is not correct for this action")]
    WrongCheckInStatus {},
}
