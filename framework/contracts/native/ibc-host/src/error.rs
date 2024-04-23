use abstract_sdk::AbstractSdkError;
use abstract_std::{
    objects::{ans_host::AnsHostError, version_control::VersionControlError},
    AbstractError,
};
use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
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
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),

    #[error("{0}")]
    VersionControlError(#[from] VersionControlError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Expected port {0} got {1} instead.")]
    ClientMismatch(String, String),

    #[error("Chain or proxy address already registered.")]
    ProxyAddressExists {},
}

impl From<cw_semver::Error> for HostError {
    fn from(err: cw_semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
