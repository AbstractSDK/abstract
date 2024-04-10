use abstract_adapter_utils::Identify;

pub const PYTH: &str = "pyth";

#[derive(Default)]
pub struct Pyth {}

impl Identify for Pyth {
    fn name(&self) -> &'static str {
        PYTH
    }

    fn is_available_on(&self, chain_name: &str) -> bool {
        chain_name == "pyth"
    }
}

#[cfg(feature = "pyth")]
mod integration {
    use crate::{msg::OracleAction, state};
    use abstract_adapter_utils::Identify;
    use abstract_oracle_standard::{OracleCommand, OracleError, OracleQuotedPrice};
    use cosmwasm_std::{Deps, StdError, Timestamp, Uint128};
    use pyth_sdk_cw::{query_price_feed, PriceIdentifier};

    use super::Pyth;

    impl OracleCommand for Pyth {
        fn get_value(&self, deps: Deps, key: String) -> Result<OracleQuotedPrice, OracleError> {
            let pyth_addr = state::ADDRESSES_OF_PROVIDERS
                .may_load(deps.storage, self.name())?
                .ok_or(OracleError::NoAddressForProvider(self.name().to_string()))?;
            let identifier = PriceIdentifier::from_hex(key.as_bytes())
                .map_err(|e| StdError::generic_err(e.to_string()))?;
            let price_feed_response = query_price_feed(&deps.querier, pyth_addr, identifier)?;
            let price = price_feed_response.price_feed.get_price_unchecked();

            let value = Uint128::new(price.price as u128);
            let last_update = Timestamp::from_seconds(price.publish_time as u64);

            Ok(OracleQuotedPrice { value, last_update })
        }
    }
}
