use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    /// The factory contract address
    pub wyndex_factory: Addr,
}
