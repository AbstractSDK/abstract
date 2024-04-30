use abstract_adapter::AdapterError;
use abstract_sdk::AbstractSdkError;
use abstract_std::{objects::ans_host::AnsHostError, AbstractError};
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CwStakingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AdapterError(#[from] AdapterError),

    #[error("{0}")]
    AbstractSdkError(#[from] AbstractSdkError),

    #[error("{0}")]
    AbstractError(#[from] AbstractError),

    #[error("{0}")]
    AssetError(#[from] AssetError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    //Ibc not supported
    #[error("IBC queries not supported.")]
    IbcQueryNotSupported,

    #[error("Staking provider {0} is not a known provider on this network.")]
    UnknownDex(String),

    #[error("Staking provider {0} is not local to this network.")]
    ForeignDex(String),

    #[error("Cw1155 is unsupported.")]
    Cw1155Unsupported,

    #[error("Can't provide liquidity less than two assets")]
    TooFewAssets {},

    #[error("Can't provide liquidity with more than {0} assets")]
    TooManyAssets(u8),

    #[error("Provided asset {0} not in pool with assets {1:?}.")]
    ArgumentMismatch(String, Vec<String>),

    #[error("Balancer pool not supported for dex {0}.")]
    BalancerNotSupported(String),

    #[error("Pair {0} on DEX {1} does not match with pair address {2}")]
    DexMismatch(String, String, String),

    #[error("Not implemented for dex {0}")]
    NotImplemented(String),

    #[error("Maximum spread {0} exceeded for dex {1}")]
    MaxSlippageAssertion(String, String),

    #[error("Unbonding period must be set for staking {0}")]
    UnbondingPeriodNotSet(String),

    #[error("Unbonding period {0} not supported for staking {1}")]
    UnbondingPeriodNotSupported(String, String),

    #[error("Pool type {0} not supported for dex {1}")]
    NotSupportedPoolType(String, String),
}
