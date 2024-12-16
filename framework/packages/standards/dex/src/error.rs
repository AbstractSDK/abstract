use abstract_adapter::AdapterError;
use abstract_sdk::AbstractSdkError;
use abstract_std::{
    objects::{ans_host::AnsHostError, DexAssetPairing},
    AbstractError,
};
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DexError {
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

    #[error("DEX {dex} is not a known dex on this network ({:?}).", chain)]
    UnknownDexOnThisPlatform { dex: String, chain: Option<String> },

    #[error("DEX {0} is not a known dex by Abstract")]
    UnknownDex(String),

    #[error("DEX {0} is not local to this network.")]
    ForeignDex(String),

    #[error("Asset type: {0} is unsupported.")]
    UnsupportedAssetType(String),

    #[error("Can't provide liquidity with less than two assets")]
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

    #[error("Message generation for IBC queries not supported.")]
    IbcMsgQuery,

    #[error("Asset pairing {} not found.", asset_pairing)]
    AssetPairingNotFound { asset_pairing: DexAssetPairing },

    #[error("Invalid Generate Message")]
    InvalidGenerateMessage,

    #[error("Pool address not specified. You need to specify it when using raw asset addresses or denom")]
    PoolAddressEmpty,

    #[error("Only account of abstract namespace can update configuration")]
    Unauthorized {},
}
