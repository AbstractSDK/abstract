use abstract_adapter::AdapterError;
use abstract_sdk::AbstractSdkError;
use abstract_std::{objects::ans_host::AnsHostError, AbstractError};
use cosmwasm_std::{
    CheckedFromRatioError, ConversionOverflowError, DecimalRangeExceeded, StdError,
};
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MoneyMarketError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    FromRatio(#[from] CheckedFromRatioError),

    #[error("{0}")]
    AbstractOs(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    AdapterError(#[from] AdapterError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    #[error("{0}")]
    ConversionOverflow(#[from] ConversionOverflowError),

    #[error("{0}")]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("MoneyMarket {0} is not a known money-market on this network.")]
    UnknownMoneyMarket(String),

    #[error("MoneyMarket {0} is not local to this network.")]
    ForeignMoneyMarket(String),

    #[error("Asset type: {0} is unsupported.")]
    UnsupportedAssetType(String),

    #[error("Provided asset {0} not acceptable in market with assets {1:?}.")]
    ArgumentMismatch(String, Vec<String>),

    #[error("Not implemented for money-market {0}")]
    NotImplemented(String),

    #[error("Message generation for IBC queries not supported.")]
    IbcMsgQuery,

    #[error("Invalid Generate Message")]
    InvalidGenerateMessage,

    #[error("Contract address not specified. You need to specify it when using raw asset addresses or denom")]
    ContractAddressEmpty,

    #[error("Only account of abstract namespace can update configuration")]
    Unauthorized {},

    #[error("Expected native asset")]
    ExpectedNative {},
}
