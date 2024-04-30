use abstract_std::{
    objects::{
        module::{Module, ModuleInfo},
        module_reference::ModuleReference,
        namespace::Namespace,
        version_control::VersionControlContract,
        AccountId,
    },
    version_control::{ModuleConfiguration, ModuleResponse, NamespaceResponse, NamespacesResponse},
};
use cosmwasm_std::Deps;

use super::{AbstractApi, ApiIdentification};
use crate::{
    cw_helpers::ApiQuery,
    features::{AbstractRegistryAccess, ModuleIdentification},
    AbstractSdkResult,
};

/// Access the Abstract Version Control and access module information.
pub trait ModuleRegistryInterface: AbstractRegistryAccess + ModuleIdentification {
    /**
        API for querying module information from the Abstract version control contract.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let mod_registry: ModuleRegistry<MockModule>  = module.module_registry(deps.as_ref()).unwrap();
        ```
    */
    fn module_registry<'a>(&'a self, deps: Deps<'a>) -> AbstractSdkResult<ModuleRegistry<Self>> {
        let vc = self.abstract_registry(deps)?;
        Ok(ModuleRegistry {
            base: self,
            deps,
            vc,
        })
    }
}

impl<T> ModuleRegistryInterface for T where T: AbstractRegistryAccess + ModuleIdentification {}

impl<'a, T: ModuleRegistryInterface> AbstractApi<T> for ModuleRegistry<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

impl<'a, T: ModuleRegistryInterface> ApiIdentification for ModuleRegistry<'a, T> {
    fn api_id() -> String {
        "ModuleRegistry".to_owned()
    }
}

#[derive(Clone)]
/**
    API for querying module information from the Abstract version control contract.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let mod_registry: ModuleRegistry<MockModule>  = module.module_registry(deps.as_ref()).unwrap();
    ```
*/
pub struct ModuleRegistry<'a, T: ModuleRegistryInterface> {
    base: &'a T,
    deps: Deps<'a>,
    vc: VersionControlContract,
}

impl<'a, T: ModuleRegistryInterface> ModuleRegistry<'a, T> {
    /// Raw query for a module reference
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
    ) -> AbstractSdkResult<ModuleReference> {
        self.vc
            .query_module_reference_raw(module_info, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Smart query for a module
    pub fn query_module(&self, module_info: ModuleInfo) -> AbstractSdkResult<Module> {
        Ok(self
            .query_modules_configs(vec![module_info])?
            .swap_remove(0)
            .module)
    }

    /// Smart query for a module config
    pub fn query_config(&self, module_info: ModuleInfo) -> AbstractSdkResult<ModuleConfiguration> {
        Ok(self
            .query_modules_configs(vec![module_info])?
            .swap_remove(0)
            .config)
    }

    /// Smart query for a modules and its configurations
    pub fn query_modules_configs(
        &self,
        infos: Vec<ModuleInfo>,
    ) -> AbstractSdkResult<Vec<ModuleResponse>> {
        self.vc
            .query_modules_configs(infos, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Queries the account that owns the namespace
    /// Is also returns the base modules of that account (AccountBase)
    pub fn query_namespace(&self, namespace: Namespace) -> AbstractSdkResult<NamespaceResponse> {
        self.vc
            .query_namespace(namespace, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Queries the namespaces owned by accounts
    pub fn query_namespaces(
        &self,
        accounts: Vec<AccountId>,
    ) -> AbstractSdkResult<NamespacesResponse> {
        self.vc
            .query_namespaces(accounts, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Queries the module info of the standalone code id
    pub fn query_standalone_info_raw(&self, code_id: u64) -> AbstractSdkResult<ModuleInfo> {
        self.vc
            .query_standalone_info_raw(code_id, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }
}
