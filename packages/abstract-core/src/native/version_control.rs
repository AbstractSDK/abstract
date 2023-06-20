//! # Version Control
//!
//! `abstract_core::version_control` stores chain-specific code-ids, addresses and an account_id map.
//!
//! ## Description
//! Code-ids and api-contract addresses are stored on this address. This data can not be changed and allows for complex factory logic.
//! Both code-ids and addresses are stored on a per-module version basis which allows users to easily upgrade their modules.
//!
//! An internal account-id store provides external verification for manager and proxy addresses.  

pub type ModuleMapEntry = (ModuleInfo, ModuleReference);

/// Contains configuration info of version control.
#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub allow_direct_module_registration: bool,
    pub namespace_limit: u32,
    pub namespace_registration_fee: cosmwasm_std::Coin,
}

pub mod state {

    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    use crate::objects::{
        account_id::AccountId,
        common_namespace::ADMIN_NAMESPACE,
        module::{ModuleInfo, Monetization},
        module_reference::ModuleReference,
        namespace::Namespace,
    };

    use super::{AccountBase, Config};

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    pub const FACTORY: Admin = Admin::new("fac");

    pub const CONFIG: Item<Config> = Item::new("cfg");

    // Modules waiting for approvals
    pub const PENDING_MODULES: Map<&ModuleInfo, ModuleReference> = Map::new("pendm");
    // We can iterate over the map giving just the prefix to get all the versions
    pub const REGISTERED_MODULES: Map<&ModuleInfo, ModuleReference> = Map::new("lib");
    // Yanked Modules
    pub const YANKED_MODULES: Map<&ModuleInfo, ModuleReference> = Map::new("yknd");
    // Modules Fee
    pub const MODULE_MONETIZATION: Map<(&Namespace, &str), Monetization> = Map::new("mod_m");

    /// Maps Account ID to the address of its core contracts
    pub const ACCOUNT_ADDRESSES: Map<AccountId, AccountBase> = Map::new("accs");
}

/// Sub indexes for namespaces.
pub struct NamespaceIndexes<'a> {
    pub account_id: MultiIndex<'a, AccountId, AccountId, &'a Namespace>,
}

impl<'a> IndexList<AccountId> for NamespaceIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<AccountId>> + '_> {
        let v: Vec<&dyn Index<AccountId>> = vec![&self.account_id];
        Box::new(v.into_iter())
    }
}

/// Primary index for namespaces.
pub fn namespaces_info<'a>() -> IndexedMap<'a, &'a Namespace, AccountId, NamespaceIndexes<'a>> {
    let indexes = NamespaceIndexes {
        account_id: MultiIndex::new(|_pk, d| *d, "namespace", "namespace_account"),
    };
    IndexedMap::new("namespace", indexes)
}

use crate::objects::{
    account_id::AccountId,
    module::{Module, ModuleInfo, ModuleStatus, Monetization},
    module_reference::ModuleReference,
    namespace::Namespace,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Coin};

use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};

/// Contains the minimal Abstract Account contract addresses.
#[cosmwasm_schema::cw_serde]
pub struct AccountBase {
    pub manager: Addr,
    pub proxy: Addr,
}

/// Version Control Instantiate Msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub allow_direct_module_registration: Option<bool>,
    pub namespace_limit: u32,
    pub namespace_registration_fee: Option<Coin>,
}

/// Version Control Execute Msg
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Remove some version of a module
    RemoveModule { module: ModuleInfo },
    /// Yank a version of a module so that it may not be installed
    /// Only callable by Admin
    YankModule { module: ModuleInfo },
    /// Propose new modules to the version registry
    /// Namespaces need to be claimed by the Account before proposing modules
    /// Once proposed, the modules need to be approved by the Admin via [`ExecuteMsg::ApproveOrRejectModules`]
    ProposeModules { modules: Vec<ModuleMapEntry> },
    /// Sets the monetization configuration for a module.
    /// The version doesn't matter here, but we keep it for compatibility purposes
    /// Only callable by namespace admin
    SetModuleMonetization {
        module_name: String,
        namespace: Namespace,
        monetization: Monetization,
    },
    /// Approve or reject modules
    /// This takes the modules in the pending_modules map and
    /// moves them to the registered_modules map or yanked_modules map
    ApproveOrRejectModules {
        approves: Vec<ModuleInfo>,
        rejects: Vec<ModuleInfo>,
    },
    /// Claim namespaces
    ClaimNamespaces {
        account_id: AccountId,
        namespaces: Vec<String>,
    },
    /// Remove namespace claims
    /// Only admin or root user can call this
    RemoveNamespaces { namespaces: Vec<String> },
    /// Register a new Account to the deployed Accounts.  
    /// Only Factory can call this
    AddAccount {
        account_id: AccountId,
        account_base: AccountBase,
    },
    /// Updates configuration of the VC contract. Available Config :
    /// 1. Whether the contract allows direct module registration
    /// 2. the number of namespaces an Account can claim
    UpdateConfig {
        allow_direct_module_registration: Option<bool>,
        namespace_limit: Option<u32>,
        namespace_registration_fee: Option<Coin>,
    },
    /// Sets a new Factory
    SetFactory { new_factory: String },
}

/// A ModuleFilter that mirrors the [`ModuleInfo`] struct.
#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub struct ModuleFilter {
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub status: Option<ModuleStatus>,
}

/// A NamespaceFilter for [`Namespaces`].
#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub struct NamespaceFilter {
    pub account_id: Option<AccountId>,
}

/// Version Control Query Msg
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Query Core of an Account
    /// Returns [`AccountBaseResponse`]
    #[returns(AccountBaseResponse)]
    AccountBase { account_id: AccountId },
    /// Queries module information
    /// Modules that are yanked are not returned
    /// Returns [`ModulesResponse`]
    #[returns(ModulesResponse)]
    Modules { infos: Vec<ModuleInfo> },
    /// Queries namespaces for an account
    /// Returns [`NamespacesResponse`]
    #[returns(NamespacesResponse)]
    Namespaces { accounts: Vec<AccountId> },
    /// Queries information about the namespace
    /// Returns [`NamespaceResponse`]
    #[returns(NamespaceResponse)]
    Namespace { namespace: Namespace },
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns [`ModulesListResponse`]
    #[returns(ModulesListResponse)]
    ModuleList {
        filter: Option<ModuleFilter>,
        start_after: Option<ModuleInfo>,
        limit: Option<u8>,
    },
    /// Returns [`NamespaceListResponse`]
    #[returns(NamespaceListResponse)]
    NamespaceList {
        filter: Option<NamespaceFilter>,
        start_after: Option<String>,
        limit: Option<u8>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct AccountBaseResponse {
    pub account_base: AccountBase,
}

#[cosmwasm_schema::cw_serde]
pub struct ModulesResponse {
    pub modules: Vec<ModuleResponse>,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleResponse {
    pub module: Module,
    pub config: ModuleConfiguration,
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub struct ModuleConfiguration {
    pub monetization: Monetization,
}

impl ModuleConfiguration {
    pub fn new(monetization: Monetization) -> Self {
        Self { monetization }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct ModulesListResponse {
    pub modules: Vec<ModuleResponse>,
}

#[cosmwasm_schema::cw_serde]
pub struct NamespaceResponse {
    pub account_id: AccountId,
    pub account_base: AccountBase,
}

#[cosmwasm_schema::cw_serde]
pub struct NamespacesResponse {
    pub namespaces: Vec<(Namespace, AccountId)>,
}

#[cosmwasm_schema::cw_serde]
pub struct NamespaceListResponse {
    pub namespaces: Vec<(Namespace, AccountId)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub factory: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
