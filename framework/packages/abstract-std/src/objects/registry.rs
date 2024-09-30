use cosmwasm_std::{Addr, Api, CanonicalAddr, QuerierWrapper, StdResult};
use thiserror::Error;

use super::{
    module::{Module, ModuleInfo},
    module_reference::ModuleReference,
    namespace::Namespace,
    AccountId,
};
use crate::{
    account::state::ACCOUNT_ID,
    native_addrs,
    registry::{
        state::{ACCOUNT_ADDRESSES, CONFIG, REGISTERED_MODULES, SERVICE_INFOS, STANDALONE_INFOS},
        Account, ModuleConfiguration, ModuleResponse, ModulesResponse, NamespaceResponse,
        NamespacesResponse, QueryMsg,
    },
};

#[derive(Error, Debug, PartialEq)]
pub enum RegistryError {
    // module not found in version registry
    #[error("Module {module} not found in version registry {registry_addr}.")]
    ModuleNotFound { module: String, registry_addr: Addr },

    // failed to query account id
    #[error("Failed to query Account id on contract {contract_addr}. Please ensure that the contract is a Manager or Proxy contract.")]
    FailedToQueryAccountId { contract_addr: Addr },

    // standalone module not found in version registry
    #[error("Standalone {code_id} not found in version registry {registry_addr}.")]
    StandaloneNotFound { code_id: u64, registry_addr: Addr },

    // unknown Account id error
    #[error("Unknown Account id {account_id} on version control {registry_addr}. Please ensure that you are using the correct Account id and version control address.")]
    UnknownAccountId {
        account_id: AccountId,
        registry_addr: Addr,
    },

    // Caller is not a valid account
    #[error("Address {0} is not the valid account address of {1}.")]
    NotAccount(Addr, AccountId),

    // Query method failed
    #[error("Query during '{method_name}' failed: {error}")]
    QueryFailed {
        method_name: String,
        error: cosmwasm_std::StdError,
    },

    // Service module not found in version registry
    #[error("Service {service_addr} not found in version registry {registry_addr}.")]
    ServiceNotFound {
        service_addr: Addr,
        registry_addr: Addr,
    },

    #[error("The provided module {0} has an invalid module reference.")]
    InvalidReference(ModuleInfo),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

/// Store the Registry contract.
#[allow(rustdoc::broken_intra_doc_links)]
/// Implements [`AbstractRegistryAccess`] (defined in abstract-sdk)
#[cosmwasm_schema::cw_serde]
pub struct RegistryContract {
    /// Address of the version control contract
    pub address: Addr,
}

impl RegistryContract {
    /// Retrieve address of the Version Control
    pub fn new(api: &dyn Api) -> StdResult<Self> {
        let address = api.addr_humanize(&CanonicalAddr::from(native_addrs::REGISTRY_ADDR))?;
        Ok(Self { address })
    }

    // Module registry

    /// Raw query for a module reference
    #[function_name::named]
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
        querier: &QuerierWrapper,
    ) -> RegistryResult<ModuleReference> {
        let module_reference = REGISTERED_MODULES
            .query(querier, self.address.clone(), module_info)
            .map_err(|error| RegistryError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;

        module_reference.ok_or_else(|| RegistryError::ModuleNotFound {
            module: module_info.to_string(),
            registry_addr: self.address.clone(),
        })
    }

    /// Smart query for a module
    pub fn query_module(
        &self,
        module_info: ModuleInfo,
        querier: &QuerierWrapper,
    ) -> RegistryResult<Module> {
        Ok(self
            .query_modules_configs(vec![module_info], querier)?
            .swap_remove(0)
            .module)
    }

    /// Smart query for a module config
    pub fn query_config(
        &self,
        module_info: ModuleInfo,
        querier: &QuerierWrapper,
    ) -> RegistryResult<ModuleConfiguration> {
        Ok(self
            .query_modules_configs(vec![module_info], querier)?
            .swap_remove(0)
            .config)
    }

    /// Smart query for a modules and its configurations
    #[function_name::named]
    pub fn query_modules_configs(
        &self,
        infos: Vec<ModuleInfo>,
        querier: &QuerierWrapper,
    ) -> RegistryResult<Vec<ModuleResponse>> {
        let ModulesResponse { modules } = querier
            .query_wasm_smart(self.address.to_string(), &QueryMsg::Modules { infos })
            .map_err(|error| RegistryError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        Ok(modules)
    }

    /// Queries the account that owns the namespace
    /// Is also returns the base modules of that account (Account)
    #[function_name::named]
    pub fn query_namespace(
        &self,
        namespace: Namespace,
        querier: &QuerierWrapper,
    ) -> RegistryResult<NamespaceResponse> {
        let namespace_response: NamespaceResponse = querier
            .query_wasm_smart(self.address.to_string(), &QueryMsg::Namespace { namespace })
            .map_err(|error| RegistryError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        Ok(namespace_response)
    }

    /// Queries the namespaces owned by accounts
    #[function_name::named]
    pub fn query_namespaces(
        &self,
        accounts: Vec<AccountId>,
        querier: &QuerierWrapper,
    ) -> RegistryResult<NamespacesResponse> {
        let namespaces_response: NamespacesResponse = querier
            .query_wasm_smart(self.address.to_string(), &QueryMsg::Namespaces { accounts })
            .map_err(|error| RegistryError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        Ok(namespaces_response)
    }

    /// Queries the module info of the standalone code id
    #[function_name::named]
    pub fn query_standalone_info_raw(
        &self,
        code_id: u64,
        querier: &QuerierWrapper,
    ) -> RegistryResult<ModuleInfo> {
        let module_info = STANDALONE_INFOS
            .query(querier, self.address.clone(), code_id)
            .map_err(|error| RegistryError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        module_info.ok_or_else(|| RegistryError::StandaloneNotFound {
            code_id,
            registry_addr: self.address.clone(),
        })
    }

    /// Queries the module info of the standalone code id
    #[function_name::named]
    pub fn query_service_info_raw(
        &self,
        service_addr: &Addr,
        querier: &QuerierWrapper,
    ) -> RegistryResult<ModuleInfo> {
        let module_info = SERVICE_INFOS
            .query(querier, self.address.clone(), service_addr)
            .map_err(|error| RegistryError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        module_info.ok_or_else(|| RegistryError::ServiceNotFound {
            service_addr: service_addr.clone(),
            registry_addr: self.address.clone(),
        })
    }

    // AccountRegistry

    /// Get self reported Account id, for checked use
    /// [`RegistryContract::account_id`]
    pub fn unchecked_account_id(
        &self,
        maybe_core_contract_addr: &Addr,
        querier: &QuerierWrapper,
    ) -> RegistryResult<AccountId> {
        ACCOUNT_ID
            .query(querier, maybe_core_contract_addr.clone())
            .map_err(|_| RegistryError::FailedToQueryAccountId {
                contract_addr: maybe_core_contract_addr.clone(),
            })
    }

    /// Get AccountId for given manager or proxy address.
    /// Also verifies that that address is indeed a manager or proxy.
    pub fn account_id(
        &self,
        maybe_account_addr: &Addr,
        querier: &QuerierWrapper,
    ) -> RegistryResult<AccountId> {
        let self_reported_account_id = self.unchecked_account_id(maybe_account_addr, querier)?;
        // now we need to verify that the account id is indeed correct
        let account = self.account(&self_reported_account_id, querier)?;
        if account.addr().ne(maybe_account_addr) {
            Err(RegistryError::FailedToQueryAccountId {
                contract_addr: maybe_account_addr.clone(),
            })
        } else {
            Ok(self_reported_account_id)
        }
    }

    /// Get the account for a given account id.
    #[function_name::named]
    pub fn account(
        &self,
        account_id: &AccountId,
        querier: &QuerierWrapper,
    ) -> RegistryResult<Account> {
        let maybe_account = ACCOUNT_ADDRESSES
            .query(querier, self.address.clone(), account_id)
            .map_err(|error| RegistryError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        maybe_account.ok_or_else(|| RegistryError::UnknownAccountId {
            account_id: account_id.clone(),
            registry_addr: self.address.clone(),
        })
    }

    /// Get namespace registration fee
    #[function_name::named]
    pub fn namespace_registration_fee(
        &self,
        querier: &QuerierWrapper,
    ) -> RegistryResult<Option<cosmwasm_std::Coin>> {
        let config = CONFIG
            .query(querier, self.address.clone())
            .map_err(|error| RegistryError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        Ok(config.namespace_registration_fee)
    }

    /// Verify if the provided account address is indeed a user.
    pub fn assert_account(
        &self,
        maybe_account: &Addr,
        querier: &QuerierWrapper,
    ) -> RegistryResult<Account> {
        let account_id = self.unchecked_account_id(maybe_account, querier)?;
        let account = self.account(&account_id, querier)?;
        if account.addr().ne(maybe_account) {
            Err(RegistryError::NotAccount(maybe_account.clone(), account_id))
        } else {
            Ok(account)
        }
    }
}
