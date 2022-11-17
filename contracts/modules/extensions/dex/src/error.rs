use abstract_extension::ExtensionError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DexError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ExtensionError(#[from] ExtensionError),

    #[error("DEX {0} is not a known dex on this network.")]
    UnknownDex(String),

    #[error("DEX {0} is not local to this network.")]
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
}
