use abstract_core::objects::{AssetEntry, DexName};
use cosmwasm_std::Addr;
use cosmwasm_std::Uint128;
use cw_storage_plus::SnapshotMap;
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("cfg");
// The sender address is used here for querying by tipper
pub const TIPPERS: SnapshotMap<(&Addr, &AssetEntry), Uint128> = SnapshotMap::new(
    "tps",
    "tps__chckp",
    "tps_chnglg",
    cw_storage_plus::Strategy::EveryBlock,
);
pub const TIP_COUNT: Item<u32> = Item::new("tip-count");
pub const TIPPER_COUNT: Map<&Addr, u32> = Map::new("tps-count");

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub desired_asset: Option<AssetEntry>,
    pub denom_asset: String,
    pub exchanges: Vec<DexName>,
}
