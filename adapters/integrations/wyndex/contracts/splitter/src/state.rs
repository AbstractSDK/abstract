use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub addresses: Vec<(Addr, Decimal)>,
    pub cw20_addresses: Vec<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");
