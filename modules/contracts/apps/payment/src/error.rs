use abstract_app::sdk::AbstractSdkError;
use abstract_app::std::AbstractError;
use abstract_app::AppError as AbstractAppError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error(transparent)]
    Asset(#[from] AssetError),

    #[error(transparent)]
    Admin(#[from] AdminError),

    #[error(transparent)]
    DappError(#[from] AbstractAppError),

    #[error("Tipper does not exist")]
    TipperDoesNotExist {},

    #[error("Desired asset does not exist on Abstract Name Service")]
    DesiredAssetDoesNotExist {},

    #[error("Dex {0} is not registered on Abstract Name Service")]
    DexNotRegistered(String),
}
