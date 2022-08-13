use abstract_api::ApiError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DexError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ApiError(#[from] ApiError),

    #[error("DEX {0} is not a known dex on this network.")]
    UnknownDex(String),

    #[error("Cw1155 is unsupported.")]
    Cw1155Unsupported,

    #[error("Can't provide liquidity less than two assets")]
    TooFewAssets {},

    #[error("Can't provide liquidity with more than {0} assets")]
    TooManyAssets(u8),

    #[error("Pair {0} on DEX {1} does not match with pair address {2}")]
    DexMismatch(String, String, String),
}
