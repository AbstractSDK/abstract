use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cosmwasm_schema::cw_serde]
pub struct Config {}

pub const CONFIG: Item<Config> = Item::new("config");

/// Map: (`creator_addr`, `task_tag`): (`task_hash`, `task_version`)
pub const ACTIVE_TASKS: Map<(Addr, String), (String, String)> = Map::new("active_tasks");

pub const TEMP_TASK_KEY: Item<(Addr, String)> = Item::new("temp_task_key");
pub const REMOVED_TASK_MANAGER_ADDR: Item<Addr> = Item::new("removed_task_manager_addr");
