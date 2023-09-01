use abstract_core::AbstractError;
use abstract_sdk::{core::abstract_ica::SimpleIcaError, AbstractSdkError};
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum HostError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("This host does not implement any custom queries")]
    NoCustomQueries,

    #[error("{0}")]
    AdminError(#[from] AdminError),

    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),

    #[error("{0}")]
    SimpleIca(#[from] SimpleIcaError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Invalid reply id")]
    InvalidReplyId,

    #[error("A valid proxy address must be provided.")]
    MissingProxyAddress,

    #[error("Missing target proxy to send messages to.")]
    NoTarget,

    #[error("Remote account can not be created from a Local trace")]
    LocalTrace,

    #[error("Expected port {0} got {1} instead.")]
    ClientMismatch(String, String),

    #[error("Chain or proxy address already registered.")]
    ProxyAddressExists,
}

impl From<semver::Error> for HostError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
