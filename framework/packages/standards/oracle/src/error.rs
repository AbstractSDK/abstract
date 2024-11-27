use abstract_adapter::AdapterError;
use abstract_sdk::AbstractSdkError;
use abstract_std::{objects::ans_host::AnsHostError, AbstractError};
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum OracleError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    AbstractOs(#[from] AbstractError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error(transparent)]
    Asset(#[from] AssetError),

    #[error(transparent)]
    AdapterError(#[from] AdapterError),

    #[error(transparent)]
    AnsHostError(#[from] AnsHostError),

    #[error("Oracle {0} is not a known oracle on this network.")]
    UnknownOracle(String),

    #[error("Oracle {0} is not local to this network.")]
    ForeignOracle(String),

    #[error("Asset type: {0} is unsupported.")]
    UnsupportedAssetType(String),

    #[error("Can't provide liquidity with less than two assets")]
    TooFewAssets {},

    #[error("Can't provide liquidity with more than {0} assets")]
    TooManyAssets(u8),

    #[error("Provided asset {0} not in pool with assets {1:?}.")]
    ArgumentMismatch(String, Vec<String>),

    #[error("Not implemented for oracle {0}")]
    NotImplemented(String),

    #[error("Message generation for IBC queries not supported.")]
    IbcMsgQuery,

    #[error("Invalid Generate Message")]
    InvalidGenerateMessage,

    #[error("Pool address not specified. You need to specify it when using raw asset addresses or denom")]
    PoolAddressEmpty,

    #[error("Only account of abstract namespace can update configuration")]
    Unauthorized {},
}
