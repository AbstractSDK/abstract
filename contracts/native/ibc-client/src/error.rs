use abstract_sdk::os::abstract_ica::SimpleIcaError;
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum IbcClientError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    SimpleIca(#[from] SimpleIcaError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("No account for channel {0}")]
    UnregisteredChannel(String),

    #[error("remote account changed from {old} to {addr}")]
    RemoteAccountChanged { addr: String, old: String },

    #[error("packages that contain internal calls are not allowed")]
    ForbiddenInternalCall {},

    #[error("The host you are trying to connect is already connected")]
    HostAlreadyExists {},
}
