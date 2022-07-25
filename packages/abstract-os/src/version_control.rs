//! # Version Control
//!
//! `abstract_os::version_control` stores chain-specific code-ids, addresses and an os_id map.
//!
//! ## Description
//! Code-ids and api-contract addresses are stored on this address. This data can not be changed and allows for complex factory logic.
//! Both code-ids and addresses are stored on a per-module version basis which allows users to easily upgrade their modules.
//!
//! An internal os-id store provides external verification for manager and proxy addresses.  

pub mod state {
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::Map;

    use super::Core;

    pub const ADMIN: Admin = Admin::new("admin");
    pub const FACTORY: Admin = Admin::new("factory");

    // Map with composite keys
    // module name + version = code_id
    // We can iterate over the map giving just the prefix to get all the versions
    pub const MODULE_CODE_IDS: Map<(&str, &str), u64> = Map::new("module_code_ids");
    // api name + version = address
    pub const API_ADDRESSES: Map<(&str, &str), Addr> = Map::new("api_address");

    /// Maps OS ID to the address of its core contracts
    pub const OS_ADDRESSES: Map<u32, Core> = Map::new("os_core");
}

use cosmwasm_std::{Addr, Uint64};
use cw2::ContractVersion;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::objects::module::ModuleInfo;

/// Contains the minimal Abstract-OS contract addresses.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Core {
    pub manager: Addr,
    pub proxy: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Call to add a new version and code-id for a module
    AddCodeId {
        module: String,
        version: String,
        code_id: u64,
    },
    /// Remove some version of a module
    RemoveCodeId { module: String, version: String },
    /// Add a new APi
    AddApi {
        module: String,
        version: String,
        address: String,
    },
    /// Remove an API
    RemoveApi { module: String, version: String },
    /// Add a new OS to the deployed OSs.  
    /// Only Factory can call this
    AddOs {
        os_id: u32,
        manager_address: String,
        proxy_address: String,
    },
    /// Sets a new Admin
    SetAdmin { new_admin: String },
    /// Sets a new Factory
    SetFactory { new_factory: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Query Core of an OS
    /// Returns [`QueryOsCoreResponse`]
    OsCore { os_id: u32 },
    /// Queries contract code_id
    /// Returns [`QueryCodeIdResponse`]
    CodeId { module: ModuleInfo },
    /// Queries api addresses
    /// Returns [`QueryApiAddressResponse`]
    ApiAddress { module: ModuleInfo },
    /// Returns [`QueryConfigResponse`]
    Config {},
    /// Returns [`QueryCodeIdsResponse`]
    CodeIds {
        last_module: Option<ContractVersion>,
        iter_limit: Option<u8>,
    },
    /// Returns [`QueryApiAddressesResponse`]
    ApiAddresses {
        last_api_module: Option<ContractVersion>,
        iter_limit: Option<u8>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryOsCoreResponse {
    pub os_core: Core,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryCodeIdResponse {
    pub code_id: Uint64,
    pub info: ContractVersion,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryCodeIdsResponse {
    pub module_code_ids: Vec<(ContractVersion, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryApiAddressResponse {
    pub address: Addr,
    pub info: ContractVersion,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryApiAddressesResponse {
    pub api_addresses: Vec<(ContractVersion, String)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryConfigResponse {
    pub admin: String,
    pub factory: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
