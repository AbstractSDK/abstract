//! # Account Manager
//!
//! `abstract_std::manager` implements the contract interface and state lay-out.
//!
//! ## Description
//!
//! The Account manager is part of the Core Abstract Account contracts along with the `abstract_std::proxy` contract.
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

    use cosmwasm_std::{Addr, Deps};
    use cw_address_like::AddressLike;
    use cw_ownable::Ownership;
    use cw_storage_plus::{Item, Map};

    pub use crate::objects::account::ACCOUNT_ID;
    use crate::objects::{
        common_namespace::OWNERSHIP_STORAGE_KEY, gov_type::GovernanceDetails, module::ModuleId,
    };

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
        pub fn verify(
            self,
            deps: Deps,
            version_control_addr: Addr,
        ) -> Result<AccountInfo<Addr>, crate::AbstractError> {
            let governance_details = self.governance_details.verify(deps, version_control_addr)?;
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
    /// Account owner - managed by cw-ownable
    pub const OWNER: Item<Ownership<Addr>> = Item::new(OWNERSHIP_STORAGE_KEY);
    /// Enabled Abstract modules
    pub const ACCOUNT_MODULES: Map<ModuleId, Addr> = Map::new("modules");
    /// Stores the dependency relationship between modules
    /// map module -> modules that depend on module.
    pub const DEPENDENTS: Map<ModuleId, HashSet<String>> = Map::new("dependents");
    /// List of sub-accounts
    pub const SUB_ACCOUNTS: Map<u32, cosmwasm_std::Empty> = Map::new("sub_accs");
    /// Pending new governance
    pub const PENDING_GOVERNANCE: Item<GovernanceDetails<Addr>> = Item::new("pgov");
    /// Context for old adapters that are currently removing authorized addresses
    pub const REMOVE_ADAPTER_AUTHORIZED_CONTEXT: Item<u64> = Item::new("rm_a_auth");
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary};
use cw2::ContractVersion;

use self::state::AccountInfo;
use crate::{
    manager::state::SuspensionStatus,
    objects::{
        account::AccountId, gov_type::GovernanceDetails, module::ModuleInfo,
        nested_admin::TopLevelOwnerResponse, AssetEntry,
    },
};

/// Manager Migrate Msg
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

/// Manager Instantiate Msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub account_id: AccountId,
    pub owner: GovernanceDetails<String>,
    pub proxy_addr: String,
    pub version_control_address: String,
    pub module_factory_address: String,
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    // Optionally modules can be provided. They will be installed after account registration.
    pub install_modules: Vec<ModuleInstallConfig>,
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

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum UpdateSubAccountAction {
    /// Unregister sub-account
    /// It will unregister sub-account from the state
    /// Could be called only by the sub-account itself
    UnregisterSubAccount { id: u32 },
    /// Register sub-account
    /// It will register new sub-account into the state
    /// Could be called by the sub-account manager
    /// Note: since it happens after the claim by this manager state won't have spam accounts
    RegisterSubAccount { id: u32 },
}

/// Module info and init message
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub struct ModuleInstallConfig {
    pub module: ModuleInfo,
    pub init_msg: Option<Binary>,
}

impl ModuleInstallConfig {
    pub fn new(module: ModuleInfo, init_msg: Option<Binary>) -> Self {
        Self { module, init_msg }
    }
}

/// Manager execute messages
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Forward execution message to module
    #[payable]
    ExecOnModule { module_id: String, exec_msg: Binary },
    /// Update Abstract-specific configuration of the module.
    /// Only callable by the account factory or owner.
    UpdateInternalConfig(Binary),
    /// Install module using module factory, callable by Owner
    #[payable]
    InstallModules {
        // Module information and Instantiate message to instantiate the contract
        modules: Vec<ModuleInstallConfig>,
    },
    /// Uninstall a module given its ID.
    UninstallModule { module_id: String },
    /// Upgrade the module to a new version
    /// If module is `abstract::manager` then the contract will do a self-migration.
    Upgrade {
        modules: Vec<(ModuleInfo, Option<Binary>)>,
    },
    /// Creates a sub-account on the account
    #[payable]
    CreateSubAccount {
        // Name of the sub-account
        name: String,
        // Description of the account
        description: Option<String>,
        // URL linked to the account
        link: Option<String>,
        // Optionally specify a base asset for the sub-account
        base_asset: Option<AssetEntry>,
        // optionally specify a namespace for the sub-account
        namespace: Option<String>,
        // Provide list of module to install after sub-account creation
        install_modules: Vec<ModuleInstallConfig>,
        /// If `None`, will create a new local account without asserting account-id.
        ///
        /// When provided sequence in 0..2147483648 range: The tx will error
        /// When provided sequence in 2147483648..u32::MAX range: Signals use of unclaimed Account Id in this range. The tx will error if this account-id already claimed. Useful for instantiate2 address prediction.
        account_id: Option<u32>,
    },
    /// Update info
    UpdateInfo {
        name: Option<String>,
        description: Option<String>,
        link: Option<String>,
    },
    /// Proposes a new Owner
    /// The new owner has to claim ownership through the [`ExecuteMsg::UpdateOwnership`] message.
    /// The claim action can be called by the new owner directly OR by the owner of a top-level account
    /// if the new ownership structure is a sub-account.
    ProposeOwner { owner: GovernanceDetails<String> },
    /// Update account statuses
    UpdateStatus { is_suspended: Option<bool> },
    /// Actions called by internal or external sub-accounts
    UpdateSubAccount(UpdateSubAccountAction),
    /// Callback endpoint
    Callback(CallbackMsg),
}

/// Manager query messages
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
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
    /// Returns [`SubAccountIdsResponse`]
    #[returns(SubAccountIdsResponse)]
    SubAccountIds {
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    /// Returns [`TopLevelOwnerResponse`]
    #[returns(TopLevelOwnerResponse)]
    TopLevelOwner {},
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
    pub account_id: AccountId,
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

#[cosmwasm_schema::cw_serde]
pub struct SubAccountIdsResponse {
    pub sub_accounts: Vec<u32>,
}
