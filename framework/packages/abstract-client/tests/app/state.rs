
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
