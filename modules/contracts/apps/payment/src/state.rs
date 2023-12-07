use abstract_core::objects::{AssetEntry, DexName};
use cosmwasm_std::Addr;
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("cfg");
// The sender address is used here for querying by tipper
pub const TIPPERS: Map<&Addr, Tipper> = Map::new("tps");
pub const TIP_COUNT: Item<u32> = Item::new("tip-count");

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub desired_asset: Option<DesiredAsset>,
    pub exchanges: Vec<DexName>,
}

#[cosmwasm_schema::cw_serde]
pub struct DesiredAsset {
    pub asset: AssetEntry,
    pub denom: String,
}

#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct Tipper {
    pub amount: Uint128,
    pub count: u32,
}

impl Tipper {
    /// Adds `amount` to `self.amount` and increments `self.count`.
    pub fn add_tip(&mut self, amount: Uint128) {
        self.amount += amount;
        self.count += 1;
    }
}
