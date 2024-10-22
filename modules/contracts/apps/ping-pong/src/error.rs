use abstract_app::{
    objects::module::ModuleInfo, sdk::AbstractSdkError, std::AbstractError,
    AppError as AbstractAppError,
};
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

    #[error("Caller module is not a ping pong: {source_module}")]
    NotPingPong { source_module: ModuleInfo },

    #[error("Can't start ping pong with zero pongs")]
    ZeroPongs {},

    #[error("First play must be a Ping")]
    FirstPlayMustBePing {},
}
