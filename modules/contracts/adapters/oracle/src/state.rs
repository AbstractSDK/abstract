use abstract_oracle_standard::msg::Config;
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");
