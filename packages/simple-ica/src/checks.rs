pub use crate::{APP_ORDER, IBC_APP_VERSION};
use cosmwasm_std::IbcOrder;

use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum SimpleIcaError {
    #[error("Only supports unordered channels")]
    InvalidChannelOrder,

    #[error("Counterparty version must be '{0}'")]
    InvalidChannelVersion(&'static str),
}

pub fn check_order(order: &IbcOrder) -> Result<(), SimpleIcaError> {
    if order != &APP_ORDER {
        Err(SimpleIcaError::InvalidChannelOrder)
    } else {
        Ok(())
    }
}

pub fn check_version(version: &str) -> Result<(), SimpleIcaError> {
    if version != IBC_APP_VERSION {
        Err(SimpleIcaError::InvalidChannelVersion(IBC_APP_VERSION))
    } else {
        Ok(())
    }
}
