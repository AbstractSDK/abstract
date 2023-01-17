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

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;

    #[test]
    fn check_order_should_reject_ordered() {
        let actual = check_order(&IbcOrder::Ordered);
        assert_that(&actual).is_err();
    }

    #[test]
    fn check_order_should_accept_unordered() {
        let actual = check_order(&APP_ORDER);
        assert_that(&actual).is_ok();
    }

    #[test]
    fn check_version_should_reject_bad_version() {
        let actual = check_version("bad-version");
        assert_that(&actual).is_err();
    }

    #[test]
    fn check_version_should_accept_good_version() {
        let actual = check_version(IBC_APP_VERSION);
        assert_that(&actual).is_ok();
    }
}
