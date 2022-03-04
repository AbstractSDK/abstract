use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub version_control_address: Addr,
    pub module_factory_address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");

pub const ADMIN: Admin = Admin::new("admin");
pub const ROOT: Admin = Admin::new("root");
pub const OS_ID: Item<u32> = Item::new("\u{0}{5}os_id");
pub const OS_MODULES: Map<&str, Addr> = Map::new("os_modules");
