use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::Item;

pub static LUNA_DENOM: &str = "uluna";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub vault_address: CanonicalAddr,
    pub denom: String,
    pub last_balance: Uint128,
    pub last_profit: Uint128,
}

pub const CONFIG: Item<State> = Item::new("\u{0}{6}config");
pub const ADMIN: Admin = Admin::new("admin");
