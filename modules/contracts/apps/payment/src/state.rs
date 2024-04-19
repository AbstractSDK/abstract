use abstract_std::objects::{AssetEntry, DexName};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, SnapshotMap, Strategy};

pub const CONFIG: Item<Config> = Item::new("cfg");
// The sender address is used here for querying by tipper
pub const TIPPERS: SnapshotMap<(&Addr, &AssetEntry), Uint128> =
    SnapshotMap::new("tps", "tps__chckp", "tps_chnglg", Strategy::EveryBlock);
pub const TIP_COUNT: Item<u32> = Item::new("tip-count");
pub const TIPPER_COUNT: SnapshotMap<&Addr, u32> = SnapshotMap::new(
    "tps-count",
    "tps-count__chckp",
    "tps-count__chglg",
    Strategy::EveryBlock,
);

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub desired_asset: Option<AssetEntry>,
    pub denom_asset: String,
    pub exchanges: Vec<DexName>,
}
