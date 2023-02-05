//! # Version Control
//!
//! `abstract_os::version_control` stores chain-specific code-ids, addresses and an os_id map.
//!
//! ## Description
//! Code-ids and api-contract addresses are stored on this address. This data can not be changed and allows for complex factory logic.
//! Both code-ids and addresses are stored on a per-module version basis which allows users to easily upgrade their modules.
//!
//! An internal os-id store provides external verification for manager and proxy addresses.  

pub type ModuleMapEntry = (ModuleInfo, ModuleReference);

pub mod state {
    use cw_controllers::Admin;
    use cw_storage_plus::Map;

    use crate::objects::{
        common_namespace::ADMIN_NAMESPACE, module::ModuleInfo, module_reference::ModuleReference,
    };
    use crate::objects::core::OsId;

    use super::Core;

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    pub const FACTORY: Admin = Admin::new("factory");

    // We can iterate over the map giving just the prefix to get all the versions
    pub const MODULE_LIBRARY: Map<ModuleInfo, ModuleReference> = Map::new("module_lib");
    /// Maps OS ID to the address of its core contracts
    pub const OS_ADDRESSES: Map<OsId, Core> = Map::new("os_core");
}

use crate::objects::{
    module::{Module, ModuleInfo},
    module_reference::ModuleReference,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;
use crate::objects::core::OsId;

/// Contains the minimal Abstract-OS contract addresses.
#[cosmwasm_schema::cw_serde]
pub struct Core {
    pub manager: Addr,
    pub proxy: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
pub enum ExecuteMsg {
    /// Remove some version of a module
    RemoveModule { module: ModuleInfo },
    /// Add new modules
    AddModules { modules: Vec<ModuleMapEntry> },
    /// Add a new OS to the deployed OSs.  
    /// Only Factory can call this
    AddOs { os_id: OsId, core: Core },
    /// Sets a new Admin
    SetAdmin { new_admin: String },
    /// Sets a new Factory
    SetFactory { new_factory: String },
}

/// A ModuleFilter that mirrors the [`ModuleInfo`] struct.
#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub struct ModuleFilter {
    pub provider: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
pub enum QueryMsg {
    /// Query Core of an OS
    /// Returns [`OsCoreResponse`]
    #[returns(OsCoreResponse)]
    OsCore { os_id: OsId },
    /// Queries api addresses
    /// Returns [`ModulesResponse`]
    #[returns(ModulesResponse)]
    Modules { infos: Vec<ModuleInfo> },
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns [`ModulesListResponse`]
    #[returns(ModulesListResponse)]
    ModuleList {
        filter: Option<ModuleFilter>,
        page_token: Option<ModuleInfo>,
        page_size: Option<u8>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct OsCoreResponse {
    pub os_core: Core,
}

#[cosmwasm_schema::cw_serde]
pub struct ModulesResponse {
    pub modules: Vec<Module>,
}

#[cosmwasm_schema::cw_serde]
pub struct ModulesListResponse {
    pub modules: Vec<ModuleMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub factory: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
