use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::{StdError, Uint128};
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DaoProxyError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    ProxyError(#[from] abstract_proxy::error::ProxyError),

    #[error("{0}")]
    DaoDao(#[from] dao_dao_core::ContractError),
}
