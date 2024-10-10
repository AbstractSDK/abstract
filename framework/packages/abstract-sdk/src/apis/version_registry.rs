use abstract_std::{
    objects::{
        module::{Module, ModuleInfo},
        module_reference::ModuleReference,
        module_version::MODULE,
        namespace::Namespace,
        registry::RegistryContract,
        AccountId,
    },
    registry::{ModuleConfiguration, ModuleResponse, NamespaceResponse, NamespacesResponse},
};
use cosmwasm_std::{Addr, Deps, Env};

use super::AbstractApi;
use crate::{
    cw_helpers::ApiQuery,
    features::{AbstractRegistryAccess, ModuleIdentification},
    AbstractSdkError, AbstractSdkResult,
};

/// Access the Abstract Version Control and access module information.
pub trait ModuleRegistryInterface: AbstractRegistryAccess + ModuleIdentification {
    /**
        API for querying module information from the Abstract registry contract.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # use abstract_testing::prelude::*;
        # let deps = mock_dependencies();
        # let account = admin_account(deps.api);
        # let module = MockModule::new(deps.api, account);

        let mod_registry: ModuleRegistry<MockModule>  = module.module_registry(deps.as_ref()).unwrap();
        ```
    */
    fn module_registry<'a>(
        &'a self,
        deps: Deps<'a>,
        env: &Env,
    ) -> AbstractSdkResult<ModuleRegistry<Self>> {
        let vc = self.abstract_registry(deps, env)?;
        Ok(ModuleRegistry {
            base: self,
            deps,
            vc,
        })
    }
}

impl<T> ModuleRegistryInterface for T where T: AbstractRegistryAccess + ModuleIdentification {}

impl<'a, T: ModuleRegistryInterface> AbstractApi<T> for ModuleRegistry<'a, T> {
    const API_ID: &'static str = "ModuleRegistry";

    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

#[derive(Clone)]
/**
    API for querying module information from the Abstract registry contract.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # use abstract_testing::prelude::*;
    # let deps = mock_dependencies();
    # let account = admin_account(deps.api);
    # let module = MockModule::new(deps.api, account);

    let mod_registry: ModuleRegistry<MockModule>  = module.module_registry(deps.as_ref()).unwrap();
    ```
*/
pub struct ModuleRegistry<'a, T: ModuleRegistryInterface> {
    base: &'a T,
    deps: Deps<'a>,
    vc: RegistryContract,
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
    /// Is also returns the base modules of that account (Account)
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

    /// Queries the Module information for an address.
    /// This will error if the Address is not an Abstract Module (Native, Account, App, Adapter or Standalone)
    pub fn module_info(&self, address: Addr) -> AbstractSdkResult<Module> {
        // We start by testing if the address is a module
        let module_response = MODULE
            .query(&self.deps.querier, address.clone())
            .map_err(|e| AbstractSdkError::NotAModule {
                addr: address.clone(),
                err: e.to_string(),
            })?;

        // We verify the module is indeed registered inside the version registry
        let module = self.query_module(ModuleInfo::from_id(
            &module_response.module,
            module_response.version.into(),
        )?)?;

        match module.reference.clone() {
            ModuleReference::Adapter(queried_address)
            | ModuleReference::Native(queried_address)
            | ModuleReference::Service(queried_address) => {
                if queried_address == address {
                    Ok(module)
                } else {
                    Err(AbstractSdkError::WrongModuleInfo {
                        addr: address.clone(),
                        module: module.to_string(),
                        err: format!("Expected address {queried_address}, got address {address}",),
                    })
                }
            }
            ModuleReference::App(queried_code_id)
            | ModuleReference::Standalone(queried_code_id)
            | ModuleReference::Account(queried_code_id) => {
                let request_contract = self.deps.querier.query_wasm_contract_info(&address)?;
                if queried_code_id == request_contract.code_id {
                    Ok(module)
                } else {
                    Err(AbstractSdkError::WrongModuleInfo {
                        addr: address,
                        module: module.to_string(),
                        err: format!(
                            "Expected code_id {queried_code_id}, got code_id {}",
                            request_contract.code_id
                        ),
                    })
                }
            }
            _ => Err(AbstractSdkError::NotAModule {
                addr: address,
                err: "got an un-implemented module reference".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{apis::traits::test::abstract_api_test, mock_module::mock_module_setup};

    use abstract_testing::prelude::*;

    #[coverage_helper::test]
    fn abstract_api() {
        let (deps, _, app) = mock_module_setup();
        let env = mock_env_validated(deps.api);
        let module_registry = app.module_registry(deps.as_ref(), &env).unwrap();

        abstract_api_test(module_registry);
    }
}
