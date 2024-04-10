use abstract_adapter_utils::Identify;
use cosmwasm_std::{Timestamp, Uint128};

use crate::OracleError;

/// # OracleCommand
/// ensures Oracle adapters support the expected functionality.
///
/// Implements the usual Oracle operations.
pub trait OracleCommand: Identify {
    /// Get value in USD for the given key and when it was updated
    fn get_value(&self, key: String) -> Result<OracleQuotedPrice, OracleError>;
}

pub struct OracleQuotedPrice {
    pub value: Uint128,
    pub last_update: Timestamp,
}
