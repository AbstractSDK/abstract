use abstract_adapter_utils::identity::Identify;
use abstract_sdk::feature_objects::AnsHost;
use cosmwasm_std::{Deps, Env};

use crate::error::OracleError;
use crate::msg::{PriceResponse, Seconds};

/// # OracleCommand
/// ensures DEX adapters support the expected functionality.
///
/// Implements the usual DEX operations.
pub trait OracleCommand: Identify {
    /// Return oracle price given pair id
    fn price(
        &self,
        deps: Deps,
        env: &Env,
        ans_host: &AnsHost,
        price_id: String,
        no_older_than: Seconds,
    ) -> Result<PriceResponse, OracleError>;
}
