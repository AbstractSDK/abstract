//! # Registry
//!
//! `abstract_std::registry` stores chain-specific code-ids, addresses and an account_id map.
//!
//! ## Description
//! Code-ids and api-contract addresses are stored on this address. This data can not be changed and allows for complex factory logic.
//! Both code-ids and addresses are stored on a per-module version basis which allows users to easily upgrade their modules.
//!
//! An internal account-id store provides external verification for accounts.  

pub type ModuleMapEntry = (ModuleInfo, ModuleReference);

/// Contains configuration info of registry.
#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub security_enabled: bool,
    pub namespace_registration_fee: Option<Coin>,
}

pub mod state {
    use cw_storage_plus::{Item, Map};

    use super::{Account, Config, ModuleConfiguration, ModuleDefaultConfiguration};
    use crate::objects::{
        account::{AccountId, AccountSequence},
        module::ModuleInfo,
        module_reference::ModuleReference,
        namespace::Namespace,
        storage_namespaces::{self},
    };

    pub const CONFIG: Item<Config> = Item::new(storage_namespaces::CONFIG_STORAGE_KEY);

    // Modules waiting for approvals
    pub const PENDING_MODULES: Map<&ModuleInfo, ModuleReference> =
        Map::new(storage_namespaces::registry::PENDING_MODULES);
    // We can iterate over the map giving just the prefix to get all the versions
    pub const REGISTERED_MODULES: Map<&ModuleInfo, ModuleReference> =
        Map::new(storage_namespaces::registry::REGISTERED_MODULES);
    // Reverse map for module info of standalone modules
    pub const STANDALONE_INFOS: Map<u64, ModuleInfo> =
        Map::new(storage_namespaces::registry::STANDALONE_INFOS);
    // Reverse map for module info of service modules
    pub const SERVICE_INFOS: Map<&cosmwasm_std::Addr, ModuleInfo> =
        Map::new(storage_namespaces::registry::SERVICE_INFOS);
    // Yanked Modules
    pub const YANKED_MODULES: Map<&ModuleInfo, ModuleReference> =
        Map::new(storage_namespaces::registry::YANKED_MODULES);
    // Modules Configuration
    pub const MODULE_CONFIG: Map<&ModuleInfo, ModuleConfiguration> =
        Map::new(storage_namespaces::registry::MODULE_CONFIG);
    // Modules Default Configuration
    pub const MODULE_DEFAULT_CONFIG: Map<(&Namespace, &str), ModuleDefaultConfiguration> =
        Map::new(storage_namespaces::registry::MODULE_DEFAULT_CONFIG);
    /// Maps Account ID to the address of its core contracts
    pub const ACCOUNT_ADDRESSES: Map<&AccountId, Account> =
        Map::new(storage_namespaces::registry::ACCOUNT_ADDRESSES);
    /// Account sequences
    pub const LOCAL_ACCOUNT_SEQUENCE: Item<AccountSequence> =
        Item::new(storage_namespaces::registry::LOCAL_ACCOUNT_SEQUENCE);
    pub const NAMESPACES: Map<&Namespace, AccountId> =
        Map::new(storage_namespaces::registry::NAMESPACES);
    pub const REV_NAMESPACES: Map<&AccountId, Namespace> =
        Map::new(storage_namespaces::registry::REV_NAMESPACES);
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Api, Coin, Storage};
use cw_clearable::Clearable;

use self::state::{MODULE_CONFIG, MODULE_DEFAULT_CONFIG};
use crate::objects::{
    account::AccountId,
    module::{Module, ModuleInfo, ModuleMetadata, ModuleStatus, Monetization},
    module_reference::ModuleReference,
    namespace::Namespace,
};

/// Contains the minimal Abstract Account contract addresses.
#[cosmwasm_schema::cw_serde]
pub struct Account<T = Addr>(T);

impl<T> Account<T> {
    pub fn new(addr: T) -> Self {
        Self(addr)
    }
}

impl Account<String> {
    pub fn verify(self, api: &dyn Api) -> cosmwasm_std::StdResult<Account<Addr>> {
        let addr = api.addr_validate(&self.0)?;
        Ok(Account(addr))
    }
}

impl Account {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn into_addr(self) -> Addr {
        self.0
    }
}

impl From<Account<Addr>> for Account<String> {
    fn from(addr: Account<Addr>) -> Self {
        Account(addr.0.to_string())
    }
}

/// Registry Instantiate Msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    /// allows users to directly register modules without going through approval
    /// Also allows them to change the module reference of an existing module
    /// Also allows to claim namespaces permisionlessly
    /// SHOULD ONLY BE `true` FOR TESTING
    pub security_enabled: Option<bool>,
    pub namespace_registration_fee: Option<Coin>,
}

/// Registry Execute Msg
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
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
    /// Sets the metadata configuration for a module.
    /// Only callable by namespace admin
    UpdateModuleConfiguration {
        module_name: String,
        namespace: Namespace,
        update_module: UpdateModule,
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
    /// Forgo namespace claims
    /// Only admin or root user can call this
    ForgoNamespace { namespaces: Vec<String> },
    /// Register a new Account to the deployed Accounts.
    /// Claims namespace if provided.  
    /// Only new accounts can call this.
    AddAccount {
        namespace: Option<String>,
        creator: String,
    },
    /// Updates configuration of the Registry contract
    UpdateConfig {
        /// Whether the contract allows direct module registration
        security_enabled: Option<bool>,
        /// The fee charged when registering a namespace
        namespace_registration_fee: Option<Clearable<Coin>>,
    },
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum UpdateModule {
    /// Updates the default metadata for the module
    Default { metadata: ModuleMetadata },
    /// Update configuration for specified version
    Versioned {
        /// Module version
        version: String,
        /// Update the metadata for this version
        metadata: Option<ModuleMetadata>,
        /// Update the monetization for this version
        monetization: Option<Monetization>,
        /// Update the init_funds for this version
        instantiation_funds: Option<Vec<Coin>>,
    },
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

/// Registry Query Msg
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Query Core of Accounts
    /// Returns [`AccountsResponse`]
    #[returns(AccountsResponse)]
    Accounts { account_ids: Vec<AccountId> },
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
    /// Returns [`AccountListResponse`]
    #[returns(AccountListResponse)]
    AccountList {
        start_after: Option<AccountId>,
        limit: Option<u8>,
    },
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
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountListResponse {
    pub accounts: Vec<(AccountId, Account)>,
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
#[derive(Default)]
pub struct ModuleConfiguration {
    pub monetization: Monetization,
    pub metadata: Option<ModuleMetadata>,
    pub instantiation_funds: Vec<Coin>,
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub struct ModuleDefaultConfiguration {
    pub metadata: ModuleMetadata,
}

impl ModuleDefaultConfiguration {
    pub fn new(metadata: ModuleMetadata) -> Self {
        Self { metadata }
    }
}

impl ModuleConfiguration {
    pub fn new(
        monetization: Monetization,
        metadata: Option<ModuleMetadata>,
        instantiation_funds: Vec<Coin>,
    ) -> Self {
        Self {
            monetization,
            metadata,
            instantiation_funds,
        }
    }

    pub fn from_storage(
        storage: &dyn Storage,
        module: &ModuleInfo,
    ) -> cosmwasm_std::StdResult<Self> {
        let mut mod_cfg = MODULE_CONFIG.may_load(storage, module)?.unwrap_or_default();

        if mod_cfg.metadata.is_none() {
            // Destructure so we notice any field changes at compile time
            if let Some(ModuleDefaultConfiguration { metadata }) =
                MODULE_DEFAULT_CONFIG.may_load(storage, (&module.namespace, &module.name))?
            {
                mod_cfg.metadata = Some(metadata);
            }
        }

        Ok(mod_cfg)
    }
}

#[cosmwasm_schema::cw_serde]
pub struct ModulesListResponse {
    pub modules: Vec<ModuleResponse>,
}

#[cosmwasm_schema::cw_serde]
pub enum NamespaceResponse {
    Claimed(NamespaceInfo),
    Unclaimed {},
}

impl NamespaceResponse {
    pub fn unwrap(self) -> NamespaceInfo {
        match self {
            NamespaceResponse::Claimed(info) => info,
            NamespaceResponse::Unclaimed {} => {
                panic!("called `NamespaceResponse::unwrap()` on a `Unclaimed` value")
            }
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct NamespaceInfo {
    pub account_id: AccountId,
    pub account: Account,
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
    pub security_enabled: bool,
    pub namespace_registration_fee: Option<Coin>,
    pub local_account_sequence: u32,
}

#[cosmwasm_schema::cw_serde]
pub enum MigrateMsg {
    /// Migrating from blob contract
    Instantiate(InstantiateMsg),
    /// Migrating from previous version
    Migrate {},
}
