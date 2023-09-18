use abstract_app::AppError as AbstractAppError;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use croncat_integration_utils::error::CronCatContractError;
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

    #[error("{0}")]
    CronCatContractError(#[from] CronCatContractError),

    #[error("Unable to get croncat version")]
    UnknownVersion {},

    #[error("Task already exists {task_tag}")]
    TaskAlreadyExists { task_tag: String },
}
