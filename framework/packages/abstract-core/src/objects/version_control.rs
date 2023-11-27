use cosmwasm_std::{Addr, QuerierWrapper, StdError, StdResult};

use crate::version_control::{
    state::{ACCOUNT_ADDRESSES, CONFIG, REGISTERED_MODULES, STANDALONE_INFOS},
    AccountBase, ModuleConfiguration, ModuleResponse, ModulesResponse, NamespaceResponse, QueryMsg,
};

use super::{
    account::ACCOUNT_ID,
    module::{Module, ModuleInfo},
    module_reference::ModuleReference,
    namespace::Namespace,
    AccountId,
};

/// Store the Version Control contract.
/// Implements [`AbstractRegistryAccess`]
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
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
        querier: &QuerierWrapper,
    ) -> StdResult<ModuleReference> {
        let module_reference =
            REGISTERED_MODULES.query(querier, self.address.clone(), module_info)?;
        module_reference.ok_or_else(|| {
            StdError::generic_err(format!(
                "Module {} not found in version registry {}.",
                module_info.to_string(),
                self.address
            ))
        })
    }

    /// Smart query for a module
    pub fn query_module(
        &self,
        module_info: ModuleInfo,
        querier: &QuerierWrapper,
    ) -> StdResult<Module> {
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
    ) -> StdResult<ModuleConfiguration> {
        Ok(self
            .query_modules_configs(vec![module_info], querier)?
            .swap_remove(0)
            .config)
    }

    /// Smart query for a modules and its configurations
    pub fn query_modules_configs(
        &self,
        infos: Vec<ModuleInfo>,
        querier: &QuerierWrapper,
    ) -> StdResult<Vec<ModuleResponse>> {
        let ModulesResponse { modules } =
            querier.query_wasm_smart(self.address.to_string(), &QueryMsg::Modules { infos })?;
        Ok(modules)
    }

    /// Queries the account that owns the namespace
    /// Is also returns the base modules of that account (AccountBase)
    pub fn query_namespace(
        &self,
        namespace: Namespace,
        querier: &QuerierWrapper,
    ) -> StdResult<NamespaceResponse> {
        let namespace_response: NamespaceResponse = querier
            .query_wasm_smart(self.address.to_string(), &QueryMsg::Namespace { namespace })?;
        Ok(namespace_response)
    }

    /// Queries the module info of the standalone code id
    pub fn query_standalone_info_raw(
        &self,
        code_id: u64,
        querier: &QuerierWrapper,
    ) -> StdResult<ModuleInfo> {
        let module_info = STANDALONE_INFOS.query(&querier, self.address.clone(), code_id)?;
        module_info.ok_or_else(|| {
            StdError::generic_err(format!(
                "Standalone {} not found in version registry {}.",
                code_id, self.address
            ))
        })
    }

    // AccountRegistry

    /// Get AccountId for given manager or proxy address.
    pub fn account_id(
        &self,
        maybe_core_contract_addr: &Addr,
        querier: &QuerierWrapper,
    ) -> StdResult<AccountId> {
        ACCOUNT_ID
            .query(&querier, maybe_core_contract_addr.clone())
            .map_err(|_| StdError::generic_err(format!("Failed to query Account id on contract {}. Please ensure that the contract is a Manager or Proxy contract.", maybe_core_contract_addr)))
    }

    /// Get the account base for a given account id.
    pub fn account_base(
        &self,
        account_id: &AccountId,
        querier: &QuerierWrapper,
    ) -> StdResult<AccountBase> {
        let maybe_account = ACCOUNT_ADDRESSES.query(querier, self.address.clone(), account_id)?;
        maybe_account.ok_or_else(|| StdError::generic_err(format!("Unknown Account id {} on version control {}. Please ensure that you are using the correct Account id and version control address.", account_id, self.address)))
    }

    /// Get namespace registration fee
    pub fn namespace_registration_fee(
        &self,
        querier: &QuerierWrapper,
    ) -> StdResult<Option<cosmwasm_std::Coin>> {
        let config = CONFIG.query(querier, self.address.clone())?;
        if config.namespace_registration_fee.amount.is_zero() {
            Ok(None)
        } else {
            Ok(Some(config.namespace_registration_fee))
        }
    }

        // /// Verify if the provided manager address is indeed a user.
        // pub fn assert_manager(&self, maybe_manager: &Addr, querier: &QuerierWrapper) -> StdResult<AccountBase> {
        //     let account_id = self.account_id(maybe_manager, querier)?;
        //     let account_base = self.account_base(&account_id, querier)?;
        //     if account_base.manager.ne(maybe_manager) {
        //         Err(AbstractSdkError::NotManager(
        //             maybe_manager.clone(),
        //             account_id,
        //         ))
        //     } else {
        //         Ok(account_base)
        //     }
        // }
    
        // /// Verify if the provided proxy address is indeed a user.
        // pub fn assert_proxy(&self, maybe_proxy: &Addr) -> AbstractSdkResult<AccountBase> {
        //     let account_id = self.account_id(maybe_proxy)?;
        //     let account_base = self.account_base(&account_id)?;
        //     if account_base.proxy.ne(maybe_proxy) {
        //         Err(AbstractSdkError::NotProxy(maybe_proxy.clone(), account_id))
        //     } else {
        //         Ok(account_base)
        //     }
        // }
}
