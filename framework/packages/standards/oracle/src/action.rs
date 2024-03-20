#![warn(missing_docs)]
//! # Oracle Adapter Action Definition
//!
use abstract_core::objects::AssetEntry;
// TODO: Do we need this object inside abstract_core?
use abstract_core::objects::price_source::UncheckedPriceSource;

/// Possible actions to perform on the Adapter
#[cosmwasm_schema::cw_serde]
pub enum OracleConfiguration {
    /// Update config for the account
    UpdateConfig {
        /// Filter the price if it wasn't updated within age seconds of the current timestamp.
        external_age_max: Option<u64>,
    },
    /// Update oracle assets for account
    UpdateAssets {
        /// Add assets for account value evaluation
        to_add: Vec<(AssetEntry, UncheckedPriceSource)>,
        /// Remove assets from account value evaluation
        to_remove: Vec<AssetEntry>,
    },
}
