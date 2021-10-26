use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub whale_token_addr: CanonicalAddr,
    pub whale_pool_addr: CanonicalAddr,
    pub anchor_money_market_addr: CanonicalAddr,
    pub aust_addr: CanonicalAddr,
    pub deposits_in_uusd: Uint128,
    pub last_deposit_in_uusd: Uint128,
    pub anchor_deposit_threshold: Uint128,
    pub anchor_withdraw_threshold: Uint128,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
