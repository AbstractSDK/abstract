use cosmwasm_std::{Addr, QuerierWrapper};
use thiserror::Error;

use super::{
    account::ACCOUNT_ID,
    module::{Module, ModuleInfo},
    module_reference::ModuleReference,
    namespace::Namespace,
    AccountId,
};
use crate::version_control::{
    state::{ACCOUNT_ADDRESSES, CONFIG, REGISTERED_MODULES, SERVICE_INFOS, STANDALONE_INFOS},
    AccountBase, ModuleConfiguration, ModuleResponse, ModulesResponse, NamespaceResponse,
    NamespacesResponse, QueryMsg,
};

#[derive(Error, Debug, PartialEq)]
pub enum VersionControlError {
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

    // caller not Manager error
    #[error("Address {0} is not the Manager of Account {1}.")]
    NotManager(Addr, AccountId),

    // caller not Proxy error
    #[error("Address {0} is not the Proxy of Account {1}.")]
    NotProxy(Addr, AccountId),

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
}

pub type VersionControlResult<T> = Result<T, VersionControlError>;

/// Store the Version Control contract.
#[allow(rustdoc::broken_intra_doc_links)]
/// Implements [`AbstractRegistryAccess`] (defined in abstract-sdk)
#[cosmwasm_schema::cw_serde]
pub struct VersionControlContract {
    /// Address of the version control contract
    pub address: Addr,
}

impl VersionControlContract {
    /// Construct a new version control feature object.
    pub fn new(address: Addr) -> Self {
        Self { address }
    }

    // Module registry

    /// Raw query for a module reference
    #[function_name::named]
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<ModuleReference> {
        let module_reference = REGISTERED_MODULES
            .query(querier, self.address.clone(), module_info)
            .map_err(|error| VersionControlError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;

        module_reference.ok_or_else(|| VersionControlError::ModuleNotFound {
            module: module_info.to_string(),
            registry_addr: self.address.clone(),
        })
    }

    /// Smart query for a module
    pub fn query_module(
        &self,
        module_info: ModuleInfo,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<Module> {
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
    ) -> VersionControlResult<ModuleConfiguration> {
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
    ) -> VersionControlResult<Vec<ModuleResponse>> {
        let ModulesResponse { modules } = querier
            .query_wasm_smart(self.address.to_string(), &QueryMsg::Modules { infos })
            .map_err(|error| VersionControlError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        Ok(modules)
    }

    /// Queries the account that owns the namespace
    /// Is also returns the base modules of that account (AccountBase)
    #[function_name::named]
    pub fn query_namespace(
        &self,
        namespace: Namespace,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<NamespaceResponse> {
        let namespace_response: NamespaceResponse = querier
            .query_wasm_smart(self.address.to_string(), &QueryMsg::Namespace { namespace })
            .map_err(|error| VersionControlError::QueryFailed {
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
    ) -> VersionControlResult<NamespacesResponse> {
        let namespaces_response: NamespacesResponse = querier
            .query_wasm_smart(self.address.to_string(), &QueryMsg::Namespaces { accounts })
            .map_err(|error| VersionControlError::QueryFailed {
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
    ) -> VersionControlResult<ModuleInfo> {
        let module_info = STANDALONE_INFOS
            .query(querier, self.address.clone(), code_id)
            .map_err(|error| VersionControlError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        module_info.ok_or_else(|| VersionControlError::StandaloneNotFound {
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
    ) -> VersionControlResult<ModuleInfo> {
        let module_info = SERVICE_INFOS
            .query(querier, self.address.clone(), service_addr)
            .map_err(|error| VersionControlError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        module_info.ok_or_else(|| VersionControlError::ServiceNotFound {
            service_addr: service_addr.clone(),
            registry_addr: self.address.clone(),
        })
    }

    // AccountRegistry

    /// Get self reported Account id, for checked use
    /// [`VersionControlContract::account_id`]
    pub fn unchecked_account_id(
        &self,
        maybe_core_contract_addr: &Addr,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<AccountId> {
        ACCOUNT_ID
            .query(querier, maybe_core_contract_addr.clone())
            .map_err(|_| VersionControlError::FailedToQueryAccountId {
                contract_addr: maybe_core_contract_addr.clone(),
            })
    }

    /// Get AccountId for given manager or proxy address.
    /// Also verifies that that address is indeed a manager or proxy.
    pub fn account_id(
        &self,
        maybe_core_contract_addr: &Addr,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<AccountId> {
        let self_reported_account_id =
            self.unchecked_account_id(maybe_core_contract_addr, querier)?;
        // now we need to verify that the account id is indeed correct
        let account_base = self.account_base(&self_reported_account_id, querier)?;
        if account_base.manager.ne(maybe_core_contract_addr)
            && account_base.proxy.ne(maybe_core_contract_addr)
        {
            Err(VersionControlError::FailedToQueryAccountId {
                contract_addr: maybe_core_contract_addr.clone(),
            })
        } else {
            Ok(self_reported_account_id)
        }
    }

    /// Get the account base for a given account id.
    #[function_name::named]
    pub fn account_base(
        &self,
        account_id: &AccountId,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<AccountBase> {
        let maybe_account = ACCOUNT_ADDRESSES
            .query(querier, self.address.clone(), account_id)
            .map_err(|error| VersionControlError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        maybe_account.ok_or_else(|| VersionControlError::UnknownAccountId {
            account_id: account_id.clone(),
            registry_addr: self.address.clone(),
        })
    }

    /// Get namespace registration fee
    #[function_name::named]
    pub fn namespace_registration_fee(
        &self,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<Option<cosmwasm_std::Coin>> {
        let config = CONFIG
            .query(querier, self.address.clone())
            .map_err(|error| VersionControlError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        Ok(config.namespace_registration_fee)
    }

    /// Verify if the provided manager address is indeed a user.
    pub fn assert_manager(
        &self,
        maybe_manager: &Addr,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<AccountBase> {
        let account_id = self.unchecked_account_id(maybe_manager, querier)?;
        let account_base = self.account_base(&account_id, querier)?;
        if account_base.manager.ne(maybe_manager) {
            Err(VersionControlError::NotManager(
                maybe_manager.clone(),
                account_id,
            ))
        } else {
            Ok(account_base)
        }
    }

    /// Verify if the provided proxy address is indeed a user.
    pub fn assert_proxy(
        &self,
        maybe_proxy: &Addr,
        querier: &QuerierWrapper,
    ) -> VersionControlResult<AccountBase> {
        let account_id = self.unchecked_account_id(maybe_proxy, querier)?;
        let account_base = self.account_base(&account_id, querier)?;
        if account_base.proxy.ne(maybe_proxy) {
            Err(VersionControlError::NotProxy(
                maybe_proxy.clone(),
                account_id,
            ))
        } else {
            Ok(account_base)
        }
    }
}
