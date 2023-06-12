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

    pub use crate::objects::account_id::ACCOUNT_ID;
    use crate::objects::common_namespace::OWNERSHIP_STORAGE_KEY;
    use crate::objects::{gov_type::GovernanceDetails, module::ModuleId};
    use cosmwasm_std::{Addr, Api};
    use cw_address_like::AddressLike;
    use cw_controllers::Admin;
    use cw_ownable::Ownership;
    use cw_storage_plus::{Item, Map};

    pub type SuspensionStatus = bool;

    /// Manager configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_address: Addr,
        pub module_factory_address: Addr,
    }

    /// Abstract Account details.
    #[cosmwasm_schema::cw_serde]
    pub struct AccountInfo<T: AddressLike = Addr> {
        pub name: String,
        pub governance_details: GovernanceDetails<T>,
        pub chain_id: String,
        pub description: Option<String>,
        pub link: Option<String>,
    }

    impl AccountInfo<String> {
        /// Check an account's info, verifying the gov details.
        pub fn verify(self, api: &dyn Api) -> Result<AccountInfo<Addr>, crate::AbstractError> {
            let governance_details = self.governance_details.verify(api)?;
            Ok(AccountInfo {
                name: self.name,
                governance_details,
                chain_id: self.chain_id,
                description: self.description,
                link: self.link,
            })
        }
    }

    impl From<AccountInfo<Addr>> for AccountInfo<String> {
        fn from(value: AccountInfo<Addr>) -> Self {
            AccountInfo {
                name: value.name,
                governance_details: value.governance_details.into(),
                chain_id: value.chain_id,
                description: value.description,
                link: value.link,
            }
        }
    }

    /// Suspension status
    pub const SUSPENSION_STATUS: Item<SuspensionStatus> = Item::new("\u{0}{12}is_suspended");
    /// Configuration
    pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
    /// Info about the Account
    pub const INFO: Item<AccountInfo<Addr>> = Item::new("\u{0}{4}info");
    /// Contract Admin
    pub const ACCOUNT_FACTORY: Admin = Admin::new("\u{0}{7}factory");
    /// Account owner - managed by cw-ownable
    pub const OWNER: Item<Ownership<Addr>> = Item::new(OWNERSHIP_STORAGE_KEY);
    /// Enabled Abstract modules
    pub const ACCOUNT_MODULES: Map<ModuleId, Addr> = Map::new("modules");
    /// Stores the dependency relationship between modules
    /// map module -> modules that depend on module.
    pub const DEPENDENTS: Map<ModuleId, HashSet<String>> = Map::new("dependents");
}

use self::state::AccountInfo;
use crate::manager::state::SuspensionStatus;
use crate::objects::{
    account_id::AccountId,
    gov_type::GovernanceDetails,
    module::{Module, ModuleInfo},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, Uint64};
use cw2::ContractVersion;

/// Manager Migrate Msg
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

/// Manager Instantiate Msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub account_id: AccountId,
    pub owner: GovernanceDetails<String>,
    pub version_control_address: String,
    pub module_factory_address: String,
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

/// Callback message to set the dependencies after module upgrades.
#[cosmwasm_schema::cw_serde]
pub struct CallbackMsg {}

/// Internal configuration actions accessible from the [`ExecuteMsg::UpdateInternalConfig`] message.
#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum InternalConfigAction {
    /// Updates the [`state::ACCOUNT_MODULES`] map
    /// Only callable by account factory or owner.
    UpdateModuleAddresses {
        to_add: Option<Vec<(String, String)>>,
        to_remove: Option<Vec<String>>,
    },
}

/// Manager execute messages
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Forward execution message to module
    ExecOnModule { module_id: String, exec_msg: Binary },
    /// Update Abstract-specific configuration of the module.
    /// Only callable by the account factory or owner.
    UpdateInternalConfig(Binary),
    /// Install module using module factory, callable by Owner
    #[cfg_attr(feature = "interface", payable)]
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
    SetOwner { owner: GovernanceDetails<String> },
    /// Update account statuses
    UpdateStatus { is_suspended: Option<bool> },
    /// Update settings for the Account, including IBC enabled, etc.
    UpdateSettings { ibc_enabled: Option<bool> },
    /// Callback endpoint
    Callback(CallbackMsg),
}

/// Manager query messages
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Query the versions of modules installed on the account given their `ids`.
    /// Returns [`ModuleVersionsResponse`]
    #[returns(ModuleVersionsResponse)]
    ModuleVersions { ids: Vec<String> },
    /// Query the addresses of modules installed on the account given their `ids`.
    /// Returns [`ModuleAddressesResponse`]
    #[returns(ModuleAddressesResponse)]
    ModuleAddresses { ids: Vec<String> },
    /// Query information of all modules installed on the account.
    /// Returns [`ModuleInfosResponse`]
    #[returns(ModuleInfosResponse)]
    ModuleInfos {
        start_after: Option<String>,
        limit: Option<u8>,
    },
    /// Query the manager's config.
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Query the Account info.
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
    pub modules: Vec<(String, Addr)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub account_id: Uint64,
    pub is_suspended: SuspensionStatus,
    pub version_control_address: Addr,
    pub module_factory_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct InfoResponse {
    pub info: AccountInfo<Addr>,
}

#[cosmwasm_schema::cw_serde]
pub struct ManagerModuleInfo {
    pub id: String,
    pub version: ContractVersion,
    pub address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleInfosResponse {
    pub module_infos: Vec<ManagerModuleInfo>,
}
