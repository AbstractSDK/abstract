use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::Frequency;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
    pub forfeit_amount: Uint128,
    pub refill_threshold: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct AccEntry {
    pub source_asset: OfferAsset,
    pub frequency: Frequency,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const ACC_LIST: Map<String, AccEntry> = Map::new("acc_list");
