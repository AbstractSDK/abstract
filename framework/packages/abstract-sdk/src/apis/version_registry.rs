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
            registry: vc,
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
    registry: RegistryContract,
}

impl<'a, T: ModuleRegistryInterface> ModuleRegistry<'a, T> {
    /// Raw query for a module reference
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
    ) -> AbstractSdkResult<ModuleReference> {
        self.registry
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
        self.registry
            .query_modules_configs(infos, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Queries the account that owns the namespace
    /// Is also returns the base modules of that account (Account)
    pub fn query_namespace(&self, namespace: Namespace) -> AbstractSdkResult<NamespaceResponse> {
        self.registry
            .query_namespace(namespace, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Queries the account_id that owns the namespace
    pub fn query_namespace_raw(
        &self,
        namespace: Namespace,
    ) -> AbstractSdkResult<Option<AccountId>> {
        self.registry
            .query_namespace_raw(namespace, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Queries the namespaces owned by accounts
    pub fn query_namespaces(
        &self,
        accounts: Vec<AccountId>,
    ) -> AbstractSdkResult<NamespacesResponse> {
        self.registry
            .query_namespaces(accounts, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Queries the module info of the standalone code id
    pub fn query_standalone_info_raw(&self, code_id: u64) -> AbstractSdkResult<ModuleInfo> {
        self.registry
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

    use abstract_std::{
        objects::{
            module::{ModuleId, Monetization},
            module_version::ModuleData,
            namespace::ABSTRACT_NAMESPACE,
            ABSTRACT_ACCOUNT_ID,
        },
        registry::ModulesResponse,
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::mock_dependencies;

    struct MockBinding {}

    impl AbstractRegistryAccess for MockBinding {
        fn abstract_registry(&self, deps: Deps, env: &Env) -> AbstractSdkResult<RegistryContract> {
            RegistryContract::new(deps.api, env).map_err(Into::into)
        }
    }

    impl ModuleIdentification for MockBinding {
        fn module_id(&self) -> ModuleId<'static> {
            ModuleId::from(TEST_MODULE_ID)
        }
    }

    #[coverage_helper::test]
    fn query_module_reference_raw() {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        deps.querier = abstract_mock_querier(deps.api);

        let binding = MockBinding {};
        let module_registry = binding.module_registry(deps.as_ref(), &env).unwrap();
        let module_reference = module_registry
            .query_module_reference_raw(
                &ModuleInfo::from_id(abstract_std::ACCOUNT, TEST_VERSION.parse().unwrap()).unwrap(),
            )
            .unwrap();
        assert_eq!(module_reference, ModuleReference::Account(1));
    }

    #[coverage_helper::test]
    fn query_namespace() {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);

        deps.querier = abstract_mock_querier_builder(deps.api)
            .with_smart_handler(&abstr.registry, |_| {
                Ok(to_json_binary(&NamespaceResponse::Unclaimed {}).unwrap())
            })
            .build();

        let binding = MockBinding {};
        let module_registry = binding.module_registry(deps.as_ref(), &env).unwrap();
        let namespace = module_registry
            .query_namespace(Namespace::new(ABSTRACT_NAMESPACE).unwrap())
            .unwrap();
        assert_eq!(namespace, NamespaceResponse::Unclaimed {});
    }

    #[coverage_helper::test]
    fn query_namespaces() {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);

        deps.querier = abstract_mock_querier_builder(deps.api)
            .with_smart_handler(&abstr.registry, |_| {
                Ok(to_json_binary(&NamespacesResponse {
                    namespaces: vec![(
                        Namespace::new(ABSTRACT_NAMESPACE).unwrap(),
                        ABSTRACT_ACCOUNT_ID,
                    )],
                })
                .unwrap())
            })
            .build();

        let binding = MockBinding {};
        let module_registry = binding.module_registry(deps.as_ref(), &env).unwrap();
        let namespaces = module_registry
            .query_namespaces(vec![ABSTRACT_ACCOUNT_ID])
            .unwrap();
        assert_eq!(
            namespaces,
            NamespacesResponse {
                namespaces: vec![(
                    Namespace::new(ABSTRACT_NAMESPACE).unwrap(),
                    ABSTRACT_ACCOUNT_ID,
                )],
            }
        );
    }

    #[coverage_helper::test]
    fn query_modules() {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let account = test_account(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);

        deps.querier = abstract_mock_querier_builder(deps.api)
            // Setup the addresses as if the Account was registered
            .account(&account, TEST_ACCOUNT_ID)
            .with_contract_item(
                &abstr.module_address,
                MODULE,
                &ModuleData {
                    module: TEST_MODULE_ID.to_owned(),
                    version: TEST_VERSION.to_owned(),
                    dependencies: vec![],
                    metadata: None,
                },
            )
            .with_smart_handler(&abstr.registry, move |_| {
                Ok(to_json_binary(&ModulesResponse {
                    modules: vec![
                        ModuleResponse {
                            module: Module {
                                info: ModuleInfo::from_id(TEST_MODULE_ID, "0.1.0".parse().unwrap())
                                    .unwrap(),
                                reference: ModuleReference::App(1),
                            },
                            config: ModuleConfiguration::new(
                                Monetization::None,
                                Some("metadata".to_owned()),
                                vec![],
                            ),
                        },
                        ModuleResponse {
                            module: Module {
                                info: ModuleInfo::from_id("test:module", "0.1.0".parse().unwrap())
                                    .unwrap(),
                                reference: ModuleReference::Standalone(2),
                            },
                            config: ModuleConfiguration::new(
                                Monetization::None,
                                Some("metadata2".to_owned()),
                                vec![],
                            ),
                        },
                    ],
                })
                .unwrap())
            })
            .build();

        let binding = MockBinding {};

        let module_info1 = ModuleInfo::from_id(TEST_MODULE_ID, "0.1.0".parse().unwrap()).unwrap();
        let module_info2 = ModuleInfo::from_id("test:module", "0.1.0".parse().unwrap()).unwrap();

        let module_registry = binding.module_registry(deps.as_ref(), &env).unwrap();
        let module = module_registry.query_module(module_info1.clone()).unwrap();
        assert_eq!(
            module,
            Module {
                info: ModuleInfo::from_id(TEST_MODULE_ID, "0.1.0".parse().unwrap()).unwrap(),
                reference: ModuleReference::App(1),
            }
        );

        let module_config = module_registry.query_config(module_info1.clone()).unwrap();
        assert_eq!(
            module_config,
            ModuleConfiguration::new(Monetization::None, Some("metadata".to_owned()), vec![])
        );

        let modules_configs = module_registry
            .query_modules_configs(vec![module_info1, module_info2])
            .unwrap();
        assert_eq!(
            modules_configs,
            vec![
                ModuleResponse {
                    module: Module {
                        info: ModuleInfo::from_id(TEST_MODULE_ID, "0.1.0".parse().unwrap())
                            .unwrap(),
                        reference: ModuleReference::App(1),
                    },
                    config: ModuleConfiguration::new(
                        Monetization::None,
                        Some("metadata".to_owned()),
                        vec![]
                    )
                },
                ModuleResponse {
                    module: Module {
                        info: ModuleInfo::from_id("test:module", "0.1.0".parse().unwrap()).unwrap(),
                        reference: ModuleReference::Standalone(2),
                    },
                    config: ModuleConfiguration::new(
                        Monetization::None,
                        Some("metadata2".to_owned()),
                        vec![]
                    )
                }
            ]
        );
        let module_info = module_registry.module_info(abstr.module_address).unwrap();
        assert_eq!(
            module_info,
            Module {
                info: ModuleInfo::from_id(TEST_MODULE_ID, "0.1.0".parse().unwrap()).unwrap(),
                reference: ModuleReference::App(1),
            }
        )
    }

    #[coverage_helper::test]
    fn abstract_api() {
        let (deps, _, app) = mock_module_setup();
        let env = mock_env_validated(deps.api);
        let module_registry = app.module_registry(deps.as_ref(), &env).unwrap();

        abstract_api_test(module_registry);
    }
}
