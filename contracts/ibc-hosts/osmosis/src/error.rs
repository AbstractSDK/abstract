use abstract_ibc_host::HostError;
use abstract_os::AbstractOsError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use dex::error::DexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum OsmoError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    DexError(#[from] DexError),

    #[error("{0}")]
    AbstractOs(#[from] AbstractOsError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    HostError(#[from] HostError),

    #[error("DEX {0} is not a known dex on this network.")]
    UnknownDex(String),

    #[error("Cw1155 is unsupported.")]
    Cw1155Unsupported,

    #[error("Can't provide liquidity less than two assets")]
    TooFewAssets {},

    #[error("Can't provide liquidity with more than {0} assets")]
    TooManyAssets(u8),

    #[error("Provided asset {0} not in pool with assets {1:?}.")]
    ArgumentMismatch(String, Vec<String>),

    #[error("Pair {0} on DEX {1} does not match with pair address {2}")]
    DexMismatch(String, String, String),
}
