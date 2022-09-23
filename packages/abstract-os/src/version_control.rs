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

    use crate::objects::module::ModuleInfo;

    use super::Core;

    pub const ADMIN: Admin = Admin::new("admin");
    pub const FACTORY: Admin = Admin::new("factory");

    // Map with composite keys
    // module name + version = code_id
    // We can iterate over the map giving just the prefix to get all the versions
    pub const MODULE_CODE_IDS: Map<ModuleInfo, u64> = Map::new("module_code_ids");
    // api name + version = address
    pub const API_ADDRESSES: Map<ModuleInfo, Addr> = Map::new("api_address");

    /// Maps OS ID to the address of its core contracts
    pub const OS_ADDRESSES: Map<u32, Core> = Map::new("os_core");
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::objects::module::ModuleInfo;

/// Contains the minimal Abstract-OS contract addresses.
#[cosmwasm_schema::cw_serde]
pub struct Core {
    pub manager: Addr,
    pub proxy: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Call to add a new version and code-id for a module
    AddCodeIds { code_ids: Vec<(ModuleInfo, u64)> },
    /// Remove some version of a module
    RemoveCodeId { module: ModuleInfo },
    /// Add a new APi
    AddApis {
        addresses: Vec<(ModuleInfo, String)>,
    },
    /// Remove an API
    RemoveApi { module: ModuleInfo },
    /// Add a new OS to the deployed OSs.  
    /// Only Factory can call this
    AddOs { os_id: u32, core: Core },
    /// Sets a new Admin
    SetAdmin { new_admin: String },
    /// Sets a new Factory
    SetFactory { new_factory: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
pub enum QueryMsg {
    /// Query Core of an OS
    /// Returns [`OsCoreResponse`]
    #[returns(OsCoreResponse)]
    OsCore { os_id: u32 },
    /// Queries contract code_id
    /// Returns [`CodeIdResponse`]
    #[returns(CodeIdResponse)]
    CodeId { module: ModuleInfo },
    /// Queries api addresses
    /// Returns [`ApiAddressResponse`]
    #[returns(ApiAddressResponse)]
    ApiAddress { module: ModuleInfo },
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns [`CodeIdsResponse`]
    #[returns(CodeIdsResponse)]
    CodeIds {
        page_token: Option<ModuleInfo>,
        page_size: Option<u8>,
    },
    /// Returns [`ApiAddressesResponse`]
    #[returns(ApiAddressesResponse)]
    ApiAddresses {
        page_token: Option<ModuleInfo>,
        page_size: Option<u8>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct OsCoreResponse {
    pub os_core: Core,
}

#[cosmwasm_schema::cw_serde]
pub struct CodeIdResponse {
    pub code_id: Uint64,
    pub info: ModuleInfo,
}

#[cosmwasm_schema::cw_serde]
pub struct CodeIdsResponse {
    pub module_code_ids: Vec<(ModuleInfo, u64)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ApiAddressResponse {
    pub address: Addr,
    pub info: ModuleInfo,
}

#[cosmwasm_schema::cw_serde]
pub struct ApiAddressesResponse {
    pub api_addresses: Vec<(ModuleInfo, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub factory: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
