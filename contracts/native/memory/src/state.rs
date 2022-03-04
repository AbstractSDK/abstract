use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Map;
use terraswap::asset::AssetInfo;

pub const ADMIN: Admin = Admin::new("admin");
// stores name and address of tokens and pairs
// LP token key: "ust_luna"
pub const ASSET_ADDRESSES: Map<&str, AssetInfo> = Map::new("assets");

// Pair key: "ust_luna_pair"
pub const CONTRACT_ADDRESSES: Map<&str, Addr> = Map::new("contracts");
