use cosmwasm_std::Addr;
use cw_asset::AssetInfo;
use cw_controllers::Admin;
use cw_storage_plus::Map;

pub const PAIR_POSTFIX: &str = "pair";

pub const ADMIN: Admin = Admin::new("admin");
// stores name and address of tokens and pairs
// LP token key: "ust_luna"
pub const ASSET_ADDRESSES: Map<&str, AssetInfo> = Map::new("assets");

// Pair key: "ust_luna_pair"
pub const CONTRACT_ADDRESSES: Map<&str, Addr> = Map::new("contracts");
