use cosmwasm_std::{OverflowError, StdError};
use cw_asset::AssetError;
use semver::Version;
use thiserror::Error;

use crate::objects::{ans_host::AnsHostError, version_control::VersionControlError};

/// Wrapper error for the Abstract framework.
#[derive(Error, Debug, PartialEq)]
pub enum AbstractError {
    #[error("Std error encountered while handling account object: {0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("cw math overflow error: {0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    VersionControlError(#[from] VersionControlError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    #[error("{0}")]
    Bech32Encode(#[from] bech32::EncodeError),

    #[error("{0}")]
    HrpError(#[from] bech32::primitives::hrp::Error),

    #[error("{0}")]
    Instantiate2AddressError(#[from] cosmwasm_std::Instantiate2AddressError),

    #[error("Semver error encountered while handling account object: {0}")]
    Semver(String),

    #[error("Entry {actual} should be formatted as {expected}")]
    EntryFormattingError { actual: String, expected: String },

    #[error("Object {object} should be formatted {expected} but is {actual}")]
    FormattingError {
        object: String,
        expected: String,
        actual: String,
    },

    #[error("Cannot downgrade contract {} from {} to {}", contract, from, to)]
    CannotDowngradeContract {
        contract: String,
        from: Version,
        to: Version,
    },

    #[error("Cannot rename contract from {} to {}", from, to)]
    ContractNameMismatch { from: String, to: String },

    #[error("Adapter {0} not installed on Account")]
    AdapterNotInstalled(String),

    #[error("App {0} not installed on Account")]
    AppNotInstalled(String),

    #[error("version for {0} in missing")]
    MissingVersion(String),

    #[error("Abstract storage object {object} errors with {msg}")]
    Storage { object: String, msg: String },

    #[error("assertion: {0}")]
    Assert(String),

    //fee error
    #[error("fee error: {0}")]
    Fee(String),

    // deposit error
    #[error("deposit error: {0}")]
    Deposit(String),

    #[error("The version or name of this module was not consistent between its stores (cw2: {cw2} and abstract module data: {module}).")]
    UnequalModuleData { cw2: String, module: String },

    #[error("Cannot Skip module version {contract} from {from} to {to}")]
    CannotSkipVersion {
        contract: String,
        from: Version,
        to: Version,
    },
}

impl From<semver::Error> for AbstractError {
    fn from(err: semver::Error) -> Self {
        AbstractError::Semver(err.to_string())
    }
}
