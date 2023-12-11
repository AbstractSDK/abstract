use crate::{
    cw_helpers::wasm_smart_query,
    features::{AbstractRegistryAccess, DepsAccess},
    AbstractSdkError, AbstractSdkResult,
};
use abstract_core::{
    objects::{
        module::{Module, ModuleInfo},
        module_reference::ModuleReference,
        namespace::Namespace,
    },
    version_control::{
        state::{REGISTERED_MODULES, STANDALONE_INFOS},
        ModuleConfiguration, ModuleResponse, ModulesResponse, NamespaceResponse, QueryMsg,
    },
};

/// Access the Abstract Version Control and access module information.
pub trait ModuleRegistryInterface: AbstractRegistryAccess + DepsAccess {
    /**
        API for querying module information from the Abstract version control contract.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let mod_registry: ModuleRegistry<MockModule>  = module.module_registry(deps.as_ref());
        ```
    */
    fn module_registry(&self) -> ModuleRegistry<Self> {
        ModuleRegistry { base: self }
    }
}

impl<T> ModuleRegistryInterface for T where T: AbstractRegistryAccess + DepsAccess {}

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

    let mod_registry: ModuleRegistry<MockModule>  = module.module_registry(deps.as_ref());
    ```
*/
pub struct ModuleRegistry<'a, T: ModuleRegistryInterface> {
    base: &'a T,
}

impl<'a, T: ModuleRegistryInterface> ModuleRegistry<'a, T> {
    /// Raw query for a module reference
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
    ) -> AbstractSdkResult<ModuleReference> {
        let registry_addr = self.base.abstract_registry()?.address;
        REGISTERED_MODULES
            .query(
                &self.base.deps().querier,
                registry_addr.clone(),
                module_info,
            )?
            .ok_or_else(|| AbstractSdkError::ModuleNotFound {
                module: module_info.to_string(),
                registry_addr,
            })
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
        let registry_addr = self.base.abstract_registry()?.address;
        let ModulesResponse { modules } = self.base.deps().querier.query(&wasm_smart_query(
            registry_addr.into_string(),
            &QueryMsg::Modules { infos },
        )?)?;
        Ok(modules)
    }

    /// Queries the account that owns the namespace
    /// Is also returns the base modules of that account (AccountBase)
    pub fn query_namespace(&self, namespace: Namespace) -> AbstractSdkResult<NamespaceResponse> {
        let registry_addr = self.base.abstract_registry()?.address;
        let namespace_response: NamespaceResponse =
            self.base.deps().querier.query(&wasm_smart_query(
                registry_addr.into_string(),
                &QueryMsg::Namespace { namespace },
            )?)?;
        Ok(namespace_response)
    }

    /// Queries the module info of the standalone code id
    pub fn query_standalone_info(&self, code_id: u64) -> AbstractSdkResult<Option<ModuleInfo>> {
        let registry_addr = self.base.abstract_registry()?.address;

        let info = STANDALONE_INFOS.query(&self.base.deps().querier, registry_addr, code_id)?;
        Ok(info)
    }
}
