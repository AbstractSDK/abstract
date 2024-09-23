//! # Account Account
//!
//! `abstract_std::account` implements the contract interface and state lay-out.
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
//!
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Binary, CosmosMsg, Empty};

use crate::objects::{
    gov_type::{GovAction, GovernanceDetails, TopLevelOwnerResponse},
    module::ModuleInfo,
    ownership::Ownership,
    AccountId,
};

use cosmwasm_std::Addr;
use cw2::ContractVersion;

use state::{AccountInfo, SuspensionStatus};

pub mod state {
    use std::collections::HashSet;

    use cosmwasm_std::Addr;
    use cw_storage_plus::{Item, Map};

    use crate::objects::{module::ModuleId, storage_namespaces, AccountId};

    pub type SuspensionStatus = bool;

    /// Manager configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_address: Addr,
        pub module_factory_address: Addr,
    }

    /// Abstract Account details.
    #[cosmwasm_schema::cw_serde]
    pub struct AccountInfo {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub link: Option<String>,
    }

    pub const WHITELISTED_MODULES: Item<WhitelistedModules> =
        Item::new(storage_namespaces::account::WHITELISTED_MODULES);

    /// Suspension status
    // TODO: Pull it inside Config as `suspended: Option<String>`, with reason of suspension inside a string?
    pub const SUSPENSION_STATUS: Item<SuspensionStatus> =
        Item::new(storage_namespaces::account::SUSPENSION_STATUS);
    /// Configuration
    pub const CONFIG: Item<Config> = Item::new(storage_namespaces::CONFIG_STORAGE_KEY);
    /// Info about the Account
    pub const INFO: Item<AccountInfo> = Item::new(storage_namespaces::account::INFO);
    /// Enabled Abstract modules
    pub const ACCOUNT_MODULES: Map<ModuleId, Addr> =
        Map::new(storage_namespaces::account::ACCOUNT_MODULES);
    /// Stores the dependency relationship between modules
    /// map module -> modules that depend on module.
    pub const DEPENDENTS: Map<ModuleId, HashSet<String>> =
        Map::new(storage_namespaces::account::DEPENDENTS);
    /// List of sub-accounts
    pub const SUB_ACCOUNTS: Map<u32, cosmwasm_std::Empty> =
        Map::new(storage_namespaces::account::SUB_ACCOUNTS);
    /// Account Id storage key
    pub const ACCOUNT_ID: Item<AccountId> = Item::new(storage_namespaces::account::ACCOUNT_ID);
    // Additional states, not listed here: cw_gov_ownable::GovOwnership, authenticators, if chain supports it

    #[cosmwasm_schema::cw_serde]
    pub struct WhitelistedModules(pub Vec<Addr>);
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

/// Account Instantiate Msg
/// https://github.com/burnt-labs/contracts/blob/main/contracts/account/src/msg.rs
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg<Authenticator = Empty> {
    pub authenticator: Option<Authenticator>,
    // pub authenticator: Option<AddAuthenticator>,
    pub account_id: Option<AccountId>,
    pub owner: GovernanceDetails<String>,
    pub namespace: Option<String>,
    // Optionally modules can be provided. They will be installed after account registration.
    pub install_modules: Vec<ModuleInstallConfig>,
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    // TODO: Compute these using instantiate2.
    pub module_factory_address: String,
    pub version_control_address: String,
}

/// Callback message to set the dependencies after module upgrades.
#[cosmwasm_schema::cw_serde]
pub struct CallbackMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg<Authenticator = Empty> {
    /// Executes the provided messages if sender is whitelisted
    ModuleAction {
        msgs: Vec<CosmosMsg<Empty>>,
    },
    /// Execute a message and forward the Response data
    ModuleActionWithData {
        msg: CosmosMsg<Empty>,
    },
    /// Execute IBC action on Client
    IbcAction {
        msg: crate::ibc_client::ExecuteMsg,
    },
    /// Queries the Abstract Ica Client with the provided action query.
    /// Provides access to different ICA implementations for different ecosystems.
    IcaAction {
        /// Query of type `abstract-ica-client::msg::QueryMsg`
        action_query_msg: Binary,
    },

    /// Forward execution message to module
    #[cw_orch(payable)]
    ExecOnModule {
        module_id: String,
        exec_msg: Binary,
    },
    /// Update Abstract-specific configuration of the module.
    /// Only callable by the account factory or owner.
    UpdateInternalConfig(InternalConfigAction),
    /// Install module using module factory, callable by Owner
    #[cw_orch(payable)]
    InstallModules {
        // Module information and Instantiate message to instantiate the contract
        modules: Vec<ModuleInstallConfig>,
    },
    /// Uninstall a module given its ID.
    UninstallModule {
        module_id: String,
    },
    /// Upgrade the module to a new version
    /// If module is `abstract::account` then the contract will do a self-migration.
    /// Self-migration is protected and only possible to the [`crate::objects::module_reference::ModuleReference::Account`] registered in Version Control
    Upgrade {
        modules: Vec<(ModuleInfo, Option<Binary>)>,
    },
    /// Creates a sub-account on the account
    #[cw_orch(payable)]
    CreateSubAccount {
        // Name of the sub-account
        name: String,
        // Description of the account
        description: Option<String>,
        // URL linked to the account
        link: Option<String>,
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
    /// Update account statuses
    UpdateStatus {
        is_suspended: Option<bool>,
    },
    /// Actions called by internal or external sub-accounts
    UpdateSubAccount(UpdateSubAccountAction),
    /// Update the contract's ownership. The `action`
    /// can propose transferring ownership to an account,
    /// accept a pending ownership transfer, or renounce the ownership
    /// of the account permanently.
    UpdateOwnership(GovAction),

    AddAuthMethod {
        add_authenticator: Authenticator,
    },
    RemoveAuthMethod {
        id: u8,
    },
    /// Callback endpoint
    Callback(CallbackMsg),
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Contains the enabled modules
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
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
    /// Query the contract's ownership information
    #[returns(Ownership<String>)]
    Ownership {},

    /// Query the pubkey associated with this account.
    // TODO: return type?
    #[returns(cosmwasm_std::Empty)]
    AuthenticatorByID { id: u8 },
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
/// Internal configuration actions accessible from the [`ExecuteMsg::UpdateInternalConfig`] message.
#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum InternalConfigAction {
    /// Updates the [`state::ACCOUNT_MODULES`] map
    /// Only callable by owner.
    UpdateModuleAddresses {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    /// Update the execution whitelist in [`state::WHITELISTED_MODULES`]
    /// Only callable by owner.
    UpdateWhitelist {
        /// Addresses to add to the Account's execution whitelist
        to_add: Vec<String>,
        /// Addresses to remove from the Account's execution whitelist
        to_remove: Vec<String>,
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
    /// Could be called by the sub-account
    RegisterSubAccount { id: u32 },
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
pub struct InfoResponse {
    pub info: AccountInfo,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountModuleInfo {
    pub id: String,
    pub version: ContractVersion,
    pub address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleInfosResponse {
    pub module_infos: Vec<AccountModuleInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct SubAccountIdsResponse {
    pub sub_accounts: Vec<u32>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub whitelisted_addresses: Vec<Addr>,
    pub account_id: AccountId,
    pub is_suspended: SuspensionStatus,
    pub version_control_address: Addr,
    pub module_factory_address: Addr,
}
