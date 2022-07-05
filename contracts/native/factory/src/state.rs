use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub version_control_contract: Addr,
    pub memory_contract: Addr,
    pub module_factory_address: Addr,
    pub subscription_address: Option<Addr>,
    pub chain_id: String,
    pub next_os_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Context {
    pub os_manager_address: Addr,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
pub const CONTEXT: Item<Context> = Item::new("\u{0}{6}context");
