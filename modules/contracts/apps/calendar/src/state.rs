use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::Time;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub price_per_minute: Uint128,
    pub denom: String,
    pub utc_offset: i32,
    pub start_time: Time,
    pub end_time: Time,
}

#[cosmwasm_schema::cw_serde]
pub struct Meeting {
    pub start_time: i64,
    pub end_time: i64,
    pub requester: Addr,
    pub amount_staked: Uint128,
}

// unix start-time of the day -> vector of meetings in that day.
pub const CALENDAR: Map<i64, Vec<Meeting>> = Map::new("calendar");
pub const CONFIG: Item<Config> = Item::new("config");
