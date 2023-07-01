use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const DEBUG: Item<bool> = Item::new("debug");
pub const OWNER: Item<Addr> = Item::new("owner");

/// for testing cosmwasm vm / storage-plus compatibility
pub const MAP: Map<String, String> = Map::new("map");
