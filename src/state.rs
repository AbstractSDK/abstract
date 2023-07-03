use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::Frequency;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
    pub dca_creation_amount: Uint128,
    pub refill_threshold: Uint128,
    pub max_spread: Decimal,
}

#[cosmwasm_schema::cw_serde]
pub struct DCAEntry {
    pub source_asset: OfferAsset,
    pub target_asset: AssetEntry,
    pub frequency: Frequency,
    pub dex: DexName,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const DCA_LIST: Map<String, DCAEntry> = Map::new("dca_list");
