use abstract_adapter::AdapterError;
use abstract_sdk::AbstractSdkError;
use abstract_std::{objects::ans_host::AnsHostError, AbstractError};
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum OracleError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    AdapterError(#[from] AdapterError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    #[error("Only account of abstract namespace can update configuration")]
    Unauthorized {},

    #[error("{0} is not a known Oracle provider on this network.")]
    UnknownProvider(String),

    #[error("{0} is not local Oracle to this network.")]
    ForeignOracle(String),

    #[error("No Address for {0} oracle provider")]
    NoAddressForProvider(String),
}
