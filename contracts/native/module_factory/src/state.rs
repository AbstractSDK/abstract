use abstract_os::{objects::module::Module, version_control::Core};
use cosmwasm_std::{Addr, Binary};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub version_control_address: Addr,
    pub memory_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Context {
    pub core: Option<Core>,
    pub module: Option<Module>,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
pub const CONTEXT: Item<Context> = Item::new("\u{0}{7}context");
pub const MODULE_INIT_BINARIES: Map<(&str, &str), Binary> = Map::new("module_init_binaries");
