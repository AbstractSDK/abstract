use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::{Map, U32Key};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Core {
    pub manager: Addr,
    pub proxy: Addr,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const FACTORY: Admin = Admin::new("factory");

// Map with composite keys
// module name + version = code_id
// We can interate over the map giving just the prefix to get all the versions
pub const MODULE_CODE_IDS: Map<(&str, &str), u64> = Map::new("module_code_ids");

// Maps OS ID to the address of its core contracts
pub const OS_ADDRESSES: Map<U32Key, Core> = Map::new("os_core");
