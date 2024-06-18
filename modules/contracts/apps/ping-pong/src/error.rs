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

    #[error("Caller module is not a ping pong: {source_module}")]
    NotPingPong { source_module: ModuleInfo },

    #[error("Can't start ping pong with zero pongs")]
    ZeroPongs {},

    #[error("Match not found for rematch")]
    NothingToRematch {},

    #[error("First play must be a Ping")]
    FirstPlayMustBePing {},
}
