use abstract_adapter::AdapterError;
use abstract_core::{objects::ans_host::AnsHostError, AbstractError};
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MoneymarketError {
    #[error("{0}")]
    Std(#[from] StdError),

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

    #[error("Moneymarket {0} is not a known money-market on this network.")]
    UnknownMoneymarket(String),

    #[error("Moneymarket {0} is not local to this network.")]
    ForeignMoneymarket(String),

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
}
