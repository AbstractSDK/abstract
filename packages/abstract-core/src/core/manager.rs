//! # Account Manager
//!
//! `abstract_core::manager` implements the contract interface and state lay-out.
//!
//! ## Description
//!
//! The Account manager is part of the Core Abstract Account contracts along with the `abstract_core::proxy` contract.
//! This contract is responsible for:
//! - Managing modules instantiation and migrations.
//! - Managing permissions.
//! - Upgrading the Account and its modules.
//! - Providing module name to address resolution.
//!
//! **The manager should be set as the contract/CosmWasm admin by default on your modules.**
//! ## Migration
//! Migrating this contract is done by calling `ExecuteMsg::Upgrade` with `abstract::manager` as module.
pub mod state {
    use std::collections::HashSet;

    pub use crate::objects::core::ACCOUNT_ID;
    use crate::objects::module::ModuleId;
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    pub type SuspensionStatus = bool;

    /// Manager configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_address: Addr,
        pub module_factory_address: Addr,
    }

    #[cosmwasm_schema::cw_serde]
    pub struct AccountInfo {
        pub name: String,
        pub governance_type: String,
        pub chain_id: String,
        pub description: Option<String>,
        pub link: Option<String>,
    }

    /// Suspension status
    pub const SUSPENSION_STATUS: Item<SuspensionStatus> = Item::new("\u{0}{12}is_suspended");
    /// Configuration
    pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
    /// Info about the Account
    pub const INFO: Item<AccountInfo> = Item::new("\u{0}{4}info");
    /// Contract Admin
    pub const ACCOUNT_FACTORY: Admin = Admin::new("\u{0}{7}factory");
    /// Account owner
    pub const OWNER: Admin = Admin::new("owner");
    /// Enabled Abstract modules
    pub const ACCOUNT_MODULES: Map<ModuleId, Addr> = Map::new("modules");
    /// Stores the dependency relationship between modules
    /// map module -> modules that depend on module.
    pub const DEPENDENTS: Map<ModuleId, HashSet<String>> = Map::new("dependents");
}

use self::state::AccountInfo;
use crate::manager::state::SuspensionStatus;
use crate::objects::{
    core::AccountId,
    module::{Module, ModuleInfo},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Binary, Uint64};
use cw2::ContractVersion;

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub account_id: AccountId,
    pub owner: String,
    pub version_control_address: String,
    pub module_factory_address: String,
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
    ExecOnModule { module_id: String, exec_msg: Binary },
    /// Updates the `ACCOUNT_MODULES` map
    /// Only callable by account factory or owner.
    UpdateModuleAddresses {
        to_add: Option<Vec<(String, String)>>,
        to_remove: Option<Vec<String>>,
    },
    /// Install module using module factory, callable by Owner
    InstallModule {
        // Module information.
        module: ModuleInfo,
        // Instantiate message used to instantiate the contract.
        init_msg: Option<Binary>,
    },
    /// Registers a module after creation.
    /// Used as a callback *only* by the Module Factory to register the module on the Account.
    RegisterModule { module_addr: String, module: Module },
    /// Uninstall a module given its ID.
    UninstallModule { module_id: String },
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
    /// Sets a new Owner
    SetOwner {
        owner: String,
        governance_type: Option<String>,
    },
    /// Update account statuses
    UpdateStatus { is_suspended: Option<bool> },
    /// Update settings for the Account, including IBC enabled, etc.
    UpdateSettings { ibc_enabled: Option<bool> },
    /// Callback endpoint
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
    pub account_id: Uint64,
    pub owner: String,
    pub is_suspended: SuspensionStatus,
    pub version_control_address: String,
    pub module_factory_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct InfoResponse {
    pub info: AccountInfo,
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
