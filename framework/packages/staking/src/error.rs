use abstract_adapter::AdapterError;
use abstract_core::objects::DexAssetPairing;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
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
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractError(#[from] AbstractSdkError),

    #[error("{0}")]
    AssetError(#[from] AssetError),

    //Ibc not supported
    #[error("IBC queries not supported.")]
    IbcQueryNotSupported,

    #[deprecated(since = "0.17.1", note = "use UnknownStakingProvider variant instead")]
    #[error("DEX {0} is not a known dex on this network.")]
    UnknownDex(String),

    #[deprecated(since = "0.17.1", note = "use ForeignStakingProvider variant instead")]
    #[error("DEX {0} is not local to this network.")]
    ForeignDex(String),

    #[error("Staking provider {0} is not a known provider on this network.")]
    UnknownStakingProvider(String),

    #[error("Staking provider {0} is not local to this network.")]
    ForeignStakingProvider(String),

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

    #[error("Not implemented for staking provider {0}")]
    NotImplemented(String),

    #[error("Maximum spread {0} exceeded for dex {1}")]
    MaxSlippageAssertion(String, String),

    #[error("Unbonding period must be set for staking {0}")]
    UnbondingPeriodNotSet(String),

    #[error("Unbonding period {0} not supported for staking {1}")]
    UnbondingPeriodNotSupported(String, String),

    #[error("Asset pairing {} not found.", asset_pairing)]
    AssetPairingNotFound { asset_pairing: DexAssetPairing },
}
