//! # OS Manager
//!
//! `abstract_os::manager` implements the contract interface and state lay-out.
//!
//! ## Description
//!
//! The OS-manager is part of the Core OS contracts along with the `abstract_os::proxy` contract.
//! This contract is responsible for:
//! - Managing modules instantiation and migrations.
//! - Managing permissions.
//! - Upgrading the OS and its modules.
//! - Providing module name to address resolution.
//!
//! **The manager should be set as the contract/CosmWasm admin by default on your modules.**
//! ## Migration
//! Migrating this contract is done by calling `ExecuteMsg::Upgrade` with `abstract::manager` as module.
pub mod state {
    use std::collections::HashSet;

    pub use crate::objects::core::OS_ID;
    use crate::ModuleId;
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    pub type Subscribed = bool;

    /// Manager configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_address: Addr,
        pub module_factory_address: Addr,
        pub subscription_address: Option<Addr>,
    }
    #[cosmwasm_schema::cw_serde]
    pub struct OsInfo {
        pub name: String,
        pub governance_type: String,
        pub chain_id: String,
        pub description: Option<String>,
        pub link: Option<String>,
    }

    /// Subscription status
    pub const STATUS: Item<Subscribed> = Item::new("\u{0}{6}status");
    /// Configuration
    pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
    /// Info about the OS
    pub const INFO: Item<OsInfo> = Item::new("\u{0}{4}info");
    /// Contract Admin
    pub const OS_FACTORY: Admin = Admin::new("\u{0}{7}factory");
    /// Root user
    pub const ROOT: Admin = Admin::new("root");
    /// Enabled Abstract modules
    pub const OS_MODULES: Map<ModuleId, Addr> = Map::new("os_modules");
    /// Stores the dependency relationship between modules
    /// map module -> modules that depend on module.
    pub const DEPENDENTS: Map<ModuleId, HashSet<String>> = Map::new("dependents");
}

use self::state::OsInfo;
use crate::objects::{
    core::OsId,
    module::{Module, ModuleInfo},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Binary, Uint64};
use cw2::ContractVersion;

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub os_id: OsId,
    pub root_user: String,
    pub version_control_address: String,
    pub module_factory_address: String,
    pub subscription_address: Option<String>,
    pub governance_type: String,
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct CallbackMsg {}

/// Execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
pub enum ExecuteMsg {
    /// Forward execution message to module
    ExecOnModule {
        module_id: String,
        exec_msg: Binary,
    },
    /// Updates the `OS_MODULES` map
    /// Only callable by os factory or root.
    UpdateModuleAddresses {
        to_add: Option<Vec<(String, String)>>,
        to_remove: Option<Vec<String>>,
    },
    /// Install module using module factory, callable by Root
    InstallModule {
        // Module information.
        module: ModuleInfo,
        // Instantiate message used to instantiate the contract.
        init_msg: Option<Binary>,
    },
    /// Registers a module after creation.
    /// Used as a callback *only* by the Module Factory to register the module on the OS.
    RegisterModule {
        module_addr: String,
        module: Module,
    },
    /// Remove a module
    RemoveModule {
        module_id: String,
    },
    /// Upgrade the module to a new version
    /// If module is `abstract::manager` then the contract will do a self-migration.
    Upgrade {
        modules: Vec<(ModuleInfo, Option<Binary>)>,
    },
    /// Update info
    UpdateInfo {
        name: Option<String>,
        description: Option<String>,
        link: Option<String>,
    },
    /// Sets a new Root
    SetRoot {
        root: String,
        governance_type: Option<String>,
    },
    /// Suspend manager contract
    SuspendOs {
        new_status: bool,
    },
    EnableIBC {
        new_status: bool,
    },
    Callback(CallbackMsg),
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
pub enum QueryMsg {
    /// Returns [`ModuleVersionsResponse`]
    #[returns(ModuleVersionsResponse)]
    ModuleVersions { ids: Vec<String> },
    /// Returns [`ModuleAddressesResponse`]
    #[returns(ModuleAddressesResponse)]
    ModuleAddresses { ids: Vec<String> },
    /// Returns [`ModuleInfosResponse`]
    #[returns(ModuleInfosResponse)]
    ModuleInfos {
        start_after: Option<String>,
        limit: Option<u8>,
    },
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns [`InfoResponse`]
    #[returns(InfoResponse)]
    Info {},
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleVersionsResponse {
    pub versions: Vec<ContractVersion>,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleAddressesResponse {
    pub modules: Vec<(String, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub root: String,
    pub version_control_address: String,
    pub module_factory_address: String,
    pub os_id: Uint64,
}

#[cosmwasm_schema::cw_serde]
pub struct InfoResponse {
    pub info: OsInfo,
}

#[cosmwasm_schema::cw_serde]
pub struct ManagerModuleInfo {
    pub id: String,
    pub version: ContractVersion,
    pub address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleInfosResponse {
    pub module_infos: Vec<ManagerModuleInfo>,
}
