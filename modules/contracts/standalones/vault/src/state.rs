use abstract_oracle_adapter::msg::Seconds;
use abstract_standalone::sdk::namespaces;
use cw_storage_plus::Item;

use crate::strategy::Strategy;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub denom_whitelist: Vec<String>,
    pub max_age: Seconds,
}

pub const CONFIG: Item<Config> = Item::new(namespaces::CONFIG_STORAGE_KEY);
pub const STRATEGIES: Item<Vec<Strategy>> = Item::new("strategies");
