use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use pandora::fee::Fee;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// State stores LP token address
/// BaseState is initialized in contract
pub struct State {
    pub liquidity_token_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// Pool stores claimable assets in vault.
/// deposit_asset is the asset which can be used to deposit into the vault.
pub struct Pool {
    pub deposit_asset: String,
    pub assets: Vec<String>,
}

pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const POOL: Item<Pool> = Item::new("\u{0}{4}pool");
pub const FEE: Item<Fee> = Item::new("\u{0}{3}fee");
