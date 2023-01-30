//! # Module
//! The Module interface provides helper functions to execute functions on other modules installed on the OS.

use super::{Dependencies, Identification};
use abstract_os::{
    api, app,
    manager::state::{ModuleId, OS_MODULES},
};
use cosmwasm_std::{
    wasm_execute, Addr, CosmosMsg, Deps, Empty, QueryRequest, StdError, StdResult, WasmQuery,
};
use cw2::{ContractVersion, CONTRACT};
use os::api::ApiRequestMsg;
use serde::{de::DeserializeOwned, Serialize};

/// Interact with other modules on the OS.
pub trait ModuleInterface: Identification + Dependencies {
    fn modules<'a>(&'a self, deps: Deps<'a>) -> Modules<Self> {
        Modules { base: self, deps }
    }
}

impl<T> ModuleInterface for T where T: Identification + Dependencies {}

#[derive(Clone)]
pub struct Modules<'a, T: ModuleInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ModuleInterface> Modules<'a, T> {
    /// Retrieve the address of an application in this OS.
    /// This should **not** be used to execute messages on an `Api`.
    /// Use `Modules::api_request(..)` instead.
    pub fn module_address(&self, module_id: ModuleId) -> StdResult<Addr> {
        let manager_addr = self.base.manager_address(self.deps)?;
        let maybe_module_addr = OS_MODULES.query(&self.deps.querier, manager_addr, module_id)?;
        let Some(module_addr) = maybe_module_addr else {
            return Err(StdError::generic_err(format!("Module {module_id} not enabled on OS.")));
        };
        Ok(module_addr)
    }

    /// Retrieve the version of an application in this OS.
    /// Note: this method makes use of the Cw2 query and may not coincide with the version of the
    /// module listed in VersionControl.
    pub fn module_version(&self, module_id: ModuleId) -> StdResult<ContractVersion> {
        let module_address = self.module_address(module_id)?;
        let req = QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: module_address.into(),
            key: CONTRACT.as_slice().into(),
        });
        self.deps.querier.query::<ContractVersion>(&req)
    }

    fn assert_module_dependency(&self, module_id: ModuleId) -> StdResult<()> {
        let is_dependency = Dependencies::dependencies(self.base)
            .iter()
            .map(|d| d.id)
            .any(|x| x == module_id);

        match is_dependency {
            true => Ok(()),
            false => Err(StdError::generic_err(format!(
                "Module {module_id} is not a dependency of this contract."
            ))),
        }
    }

    /// Construct an app request message.
    pub fn app_request<M: Serialize>(
        &self,
        app_id: ModuleId,
        message: impl Into<app::ExecuteMsg<M, Empty>>,
    ) -> StdResult<CosmosMsg> {
        self.assert_module_dependency(app_id)?;
        let app_msg: app::ExecuteMsg<M, Empty> = message.into();
        let app_address = self.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Construct an app configuation message
    pub fn app_configure(
        &self,
        app_id: ModuleId,
        message: app::BaseExecuteMsg,
    ) -> StdResult<CosmosMsg> {
        let app_msg: app::ExecuteMsg<Empty, Empty> = message.into();
        let app_address = self.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Smart query an app
    pub fn query_app<Q: Serialize, R: DeserializeOwned>(
        &self,
        app_id: ModuleId,
        message: impl Into<app::QueryMsg<Q>>,
    ) -> StdResult<R> {
        let app_msg: app::QueryMsg<Q> = message.into();
        let app_address = self.module_address(app_id)?;
        self.deps.querier.query_wasm_smart(app_address, &app_msg)
    }

    /// Interactions with Abstract APIs
    /// Construct an api request message.
    pub fn api_request<M: Serialize + Into<api::ExecuteMsg<M, Empty>>>(
        &self,
        api_id: ModuleId,
        message: M,
    ) -> StdResult<CosmosMsg> {
        self.assert_module_dependency(api_id)?;
        let api_msg = api::ExecuteMsg::<_>::App(ApiRequestMsg::new(
            Some(self.base.proxy_address(self.deps)?.into_string()),
            message,
        ));
        let api_address = self.module_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Smart query an API
    pub fn query_api<Q: Serialize, R: DeserializeOwned>(
        &self,
        api_id: ModuleId,
        message: impl Into<api::QueryMsg<Q>>,
    ) -> StdResult<R> {
        let api_msg: api::QueryMsg<Q> = message.into();
        let api_address = self.module_address(api_id)?;
        self.deps.querier.query_wasm_smart(api_address, &api_msg)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use os::objects::dependency::StaticDependency;

    use std::fmt::Debug;

    use crate::apis::test_common::*;
    use abstract_testing::TEST_MODULE_ID;

    /// Nonexistent module
    const FAKE_MODULE_ID: ModuleId = "fake_module";
    const TEST_MODULE_DEP: StaticDependency = StaticDependency::new(TEST_MODULE_ID, &[">1.0.0"]);

    impl Dependencies for MockModule {
        fn dependencies(&self) -> &[StaticDependency] {
            &[TEST_MODULE_DEP]
        }
    }

    const TEST_MODULE_ADDRESS: &str = "test_module_address";

    mod assert_module_dependency {
        use super::*;

        #[test]
        fn should_return_ok_if_dependency() {
            let deps = mock_dependencies();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let res = mods.assert_module_dependency(TEST_MODULE_ID);
            assert_that!(res).is_ok();
        }

        #[test]
        fn should_return_err_if_not_dependency() {
            let deps = mock_dependencies();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let fake_module = "lol_no_chance";
            let res = mods.assert_module_dependency(fake_module);

            assert_that!(res).is_err().matches(|e| {
                e.to_string()
                    .contains(&format!("{fake_module} is not a dependency"))
            });
        }
    }

    /// Helper to check that the method is not callable when the module is not a dependency
    fn fail_when_not_dependency_test<T: Debug>(
        modules_fn: impl FnOnce(&MockModule, Deps) -> StdResult<T>,
        fake_module: ModuleId,
    ) {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let app = MockModule::new();

        let _mods = app.modules(deps.as_ref());

        let res = modules_fn(&app, deps.as_ref());

        assert_that!(res).is_err().matches(|e| match e {
            StdError::GenericErr { msg, .. } => msg.contains(&fake_module.to_string()),
            _ => false,
        });
    }

    mod api_request {
        use super::*;
        use os::api::ApiRequestMsg;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.modules(deps);
                    mods.api_request(FAKE_MODULE_ID, MockModuleExecuteMsg {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_api_request() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let res = mods.api_request(TEST_MODULE_ID, MockModuleExecuteMsg {});

            let expected_msg: api::ExecuteMsg<_, Empty> = api::ExecuteMsg::App(ApiRequestMsg {
                proxy_address: Some(TEST_PROXY.into()),
                request: MockModuleExecuteMsg {},
            });

            assert_that!(res)
                .is_ok()
                .is_equal_to(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_MODULE_ADDRESS.into(),
                    msg: to_binary(&expected_msg).unwrap(),
                    funds: vec![],
                }));
        }
    }

    mod app_request {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.modules(deps);
                    mods.app_request(FAKE_MODULE_ID, MockModuleExecuteMsg {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_app_request() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let res = mods.app_request(TEST_MODULE_ID, MockModuleExecuteMsg {});

            let expected_msg: app::ExecuteMsg<_, Empty> =
                app::ExecuteMsg::App(MockModuleExecuteMsg {});

            assert_that!(res)
                .is_ok()
                .is_equal_to(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_MODULE_ADDRESS.into(),
                    msg: to_binary(&expected_msg).unwrap(),
                    funds: vec![],
                }));
        }
    }

    mod app_configure {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.modules(deps);
                    mods.app_configure(
                        FAKE_MODULE_ID,
                        app::BaseExecuteMsg::UpdateConfig {
                            ans_host_address: None,
                        },
                    )
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_configure_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let res = mods.app_configure(
                TEST_MODULE_ID,
                app::BaseExecuteMsg::UpdateConfig {
                    ans_host_address: Some("new_ans_addr".to_string()),
                },
            );

            let expected_msg: app::ExecuteMsg<Empty, Empty> =
                app::ExecuteMsg::Base(app::BaseExecuteMsg::UpdateConfig {
                    ans_host_address: Some("new_ans_addr".to_string()),
                });

            assert_that!(res)
                .is_ok()
                .is_equal_to(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_MODULE_ADDRESS.into(),
                    msg: to_binary(&expected_msg).unwrap(),
                    funds: vec![],
                }));
        }
    }

    mod query_api {
        use super::*;
        use os::dex::{DexQueryMsg, OfferAsset};

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.modules(deps);
                    mods.query_api::<_, Empty>(FAKE_MODULE_ID, Empty {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_api_query() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let inner_msg = DexQueryMsg::SimulateSwap {
                ask_asset: "juno".into(),
                offer_asset: OfferAsset::new("some", 69u128),
                dex: None,
            };

            let res = mods.query_api::<_, String>(TEST_MODULE_ID, inner_msg);

            assert_that!(res)
                .is_ok()
                .is_equal_to(abstract_testing::TEST_MODULE_RESPONSE.to_string());
        }
    }

    mod query_app {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.modules(deps);
                    mods.query_app::<_, Empty>(FAKE_MODULE_ID, Empty {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_app_query() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let res = mods.query_app::<_, String>(TEST_MODULE_ID, Empty {});

            assert_that!(res)
                .is_ok()
                .is_equal_to(abstract_testing::TEST_MODULE_RESPONSE.to_string());
        }
    }
}
