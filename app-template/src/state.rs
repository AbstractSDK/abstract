use cw_storage_plus::Item;

#[cosmwasm_schema::cw_serde]
pub struct Config {}

pub const CONFIG: Item<Config> = Item::new("config");
