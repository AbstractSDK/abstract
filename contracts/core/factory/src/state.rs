use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub version_control_contract: Addr,
    pub memory_contract: Addr,
    pub creation_fee: u32,
    pub next_os_id: u32,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
