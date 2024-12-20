use abstract_standalone::sdk::AbstractSdkError;
use abstract_standalone::std::AbstractError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MyStandaloneError {
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

    #[error("channel sequence number overflow, to fix: the contract admin may migrate to close and reopen the channel")]
    SequenceOverflow,
}
