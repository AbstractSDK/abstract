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
    pub allow_direct_module_registration_and_updates: bool,
    pub namespace_registration_fee: cosmwasm_std::Coin,
}

pub mod state {

    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    use crate::objects::{
        account_id::AccountId,
        common_namespace::ADMIN_NAMESPACE,
        module::{ModuleInfo, ModuleMetadata, Monetization},
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
    // Modules Metadata
    pub const MODULE_METADATA: Map<&ModuleInfo, ModuleMetadata> = Map::new("mod_meta");

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
    module::{Module, ModuleInfo, ModuleMetadata, ModuleStatus, ModuleVersion, Monetization},
    module_reference::ModuleReference,
    namespace::Namespace,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Coin, Order, Storage};
use state::MODULE_MONETIZATION;

use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};

use self::state::MODULE_METADATA;

/// Contains the minimal Abstract Account contract addresses.
#[cosmwasm_schema::cw_serde]
pub struct AccountBase {
    pub manager: Addr,
    pub proxy: Addr,
}

/// Version Control Instantiate Msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// allows users to directly register modules without going through approval
    /// Also allows them to change the module reference of an existing module
    /// SHOULD ONLY BE `true` FOR TESTING
    pub allow_direct_module_registration_and_updates: Option<bool>,
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
    /// Sets the metadata configuration for a module.
    /// Only callable by namespace admin
    /// Using Version::Latest in the [`module`] variable sets the default metadata for the module
    SetModuleMetadata {
        module: ModuleInfo,
        metadata: ModuleMetadata,
    },
    /// Approve or reject modules
    /// This takes the modules in the pending_modules map and
    /// moves them to the registered_modules map or yanked_modules map
    ApproveOrRejectModules {
        approves: Vec<ModuleInfo>,
        rejects: Vec<ModuleInfo>,
    },
    /// Claim namespaces
    ClaimNamespace {
        account_id: AccountId,
        namespace: String,
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
        allow_direct_module_registration_and_updates: Option<bool>,
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
    pub metadata: ModuleMetadata,
}

impl ModuleConfiguration {
    pub fn new(monetization: Monetization, metadata: ModuleMetadata) -> Self {
        Self {
            monetization,
            metadata,
        }
    }

    /// Loads metadata from storage for a given module
    /// This function has the following behavior
    /// 1. If available loads the metadata for the current module version
    /// 2. OR, if available, loads the metadata for the latest version of the module
    /// 3. OR, if available, loads the last metadata stored on the contract
    /// 4. OR, returns empty metadata
    fn metadata_from_storage(storage: &dyn Storage, module: &ModuleInfo) -> ModuleMetadata {
        // First we return the result if the metadata is stored for the current module
        if let Ok(metadata) = MODULE_METADATA.load(storage, module) {
            return metadata;
        }

        // Else if Version::Latest is specified, we load this description
        if let Ok(metadata) = MODULE_METADATA.load(
            storage,
            &ModuleInfo {
                namespace: module.namespace.clone(),
                name: module.name.clone(),
                version: ModuleVersion::Latest,
            },
        ) {
            return metadata;
        }

        // Else we return the latest metadata version registered with the same module
        let potential_metadata = MODULE_METADATA
            .prefix((module.namespace.clone(), module.name.clone()))
            .range(storage, None, None, Order::Descending)
            .next();
        if let Some(Ok((_key, metadata))) = potential_metadata {
            return metadata;
        }

        // Else, no metadata
        "".to_string()
    }

    pub fn from_storage(storage: &dyn Storage, module: &ModuleInfo) -> Self {
        let monetization = MODULE_MONETIZATION
            .load(storage, (&module.namespace, &module.name))
            .unwrap_or(Monetization::None);

        let metadata = ModuleConfiguration::metadata_from_storage(storage, module);

        Self {
            monetization,
            metadata,
        }
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
