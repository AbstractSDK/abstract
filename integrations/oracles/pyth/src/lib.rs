pub const AVAILABLE_CHAINS: &[&str] = &["xion", "xion-testnet"];
pub const PYTH: &str = "pyth";
use abstract_oracle_standard::Identify;

#[derive(Default)]
pub struct Pyth {}

impl Identify for Pyth {
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
    fn name(&self) -> &'static str {
        PYTH
    }
}

#[cfg(feature = "full_integration")]
use {
    abstract_oracle_standard::{msg::PriceResponse, OracleCommand, OracleError},
    abstract_sdk::feature_objects::AnsHost,
    abstract_sdk::std::objects::ContractEntry,
    cosmwasm_std::{Decimal, Deps, Env, StdError},
    pyth_sdk_cw::{query_price_feed, PriceFeedResponse, PriceIdentifier},
};

pub const XION_TEST: &str = "xion1w39ctwxxhxxc2kxarycjxj9rndn65gf8daek7ggarwh3rq3zl0lqqllnmt";

#[cfg(feature = "full_integration")]
/// Pyth oracle implementation
impl OracleCommand for Pyth {
    fn price(
        &self,
        deps: Deps,
        env: &Env,
        ans_host: &AnsHost,
        price_id: String,
        no_older_than: u64,
    ) -> Result<PriceResponse, OracleError> {
        let pyth_address = ans_host.query_contract(
            &deps.querier,
            &ContractEntry {
                protocol: PYTH.to_string(),
                contract: "oracle".to_string(),
            },
        )?;

        // We retrieve the pyth address for the current chain
        let price_feed_response: PriceFeedResponse = query_price_feed(
            &deps.querier,
            pyth_address,
            PriceIdentifier::from_hex(price_id.as_bytes())
                .map_err(|e| StdError::generic_err(format!("Wrong price id hex format, {e}")))?,
        )?;
        let price_feed = price_feed_response.price_feed;

        let current_price = price_feed
            .get_price_no_older_than(env.block.time.seconds() as i64, no_older_than)
            .ok_or_else(|| StdError::not_found("Current price is not available"))?;

        let power_ten = if current_price.expo < 0 {
            Decimal::from_ratio(1u64, 10u64).pow(
                (-current_price.expo)
                    .try_into()
                    .expect("Wrong power_of_ten logic"),
            )
        } else {
            Decimal::from_ratio(10u64, 1u64).pow(
                current_price
                    .expo
                    .try_into()
                    .expect("Wrong power_of_ten logic"),
            )
        };

        let unsigned_price: u64 = current_price
            .price
            .try_into()
            .expect("Price can't be negative");
        Ok(PriceResponse {
            price: power_ten * Decimal::from_ratio(unsigned_price, 1u64),
        })
    }
}

#[cfg(feature = "full_integration")]
impl abstract_sdk::features::ModuleIdentification for Pyth {
    fn module_id(&self) -> abstract_sdk::std::objects::module::ModuleId<'static> {
        abstract_oracle_standard::ORACLE_ADAPTER_ID
    }
}
