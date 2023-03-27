#![allow(unused)]
use crate::{AbstractSdkResult, ModuleInterface};
use abstract_core::objects::module::ModuleId;
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, Empty};
use serde::{de::DeserializeOwned, Serialize};

use abstract_core::app as msg;

/// Interact with other modules on the Account.
pub trait AppInterface: ModuleInterface {
    fn apps<'a>(&'a self, deps: Deps<'a>) -> App<Self> {
        App { base: self, deps }
    }
}

impl<T> AppInterface for T where T: ModuleInterface {}

#[derive(Clone)]
pub struct App<'a, T: AppInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: AppInterface> App<'a, T> {
    /// Construct an app request message.
    pub fn request<M: Serialize>(
        &self,
        app_id: ModuleId,
        message: impl Into<msg::ExecuteMsg<M, Empty>>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);
        modules.assert_module_dependency(app_id)?;
        let app_msg: msg::ExecuteMsg<M, Empty> = message.into();
        let app_address = modules.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Construct an app configuation message
    pub fn configure(
        &self,
        app_id: ModuleId,
        message: msg::BaseExecuteMsg,
    ) -> AbstractSdkResult<CosmosMsg> {
        let app_msg: msg::ExecuteMsg<Empty, Empty> = message.into();
        let modules = self.base.modules(self.deps);
        let app_address = modules.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Smart query an app
    pub fn query<Q: Serialize, R: DeserializeOwned>(
        &self,
        app_id: ModuleId,
        message: impl Into<msg::QueryMsg<Q>>,
    ) -> AbstractSdkResult<R> {
        let modules = self.base.modules(self.deps);
        let app_msg: msg::QueryMsg<Q> = message.into();
        let app_address = modules.module_address(app_id)?;
        self.deps
            .querier
            .query_wasm_smart(app_address, &app_msg)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use crate::mock_module::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    pub use super::*;
    /// Helper to check that the method is not callable when the module is not a dependency
    fn fail_when_not_dependency_test<T: std::fmt::Debug>(
        modules_fn: impl FnOnce(&MockModule, Deps) -> AbstractSdkResult<T>,
        fake_module: ModuleId,
    ) {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let app = MockModule::new();

        let _mods = app.apps(deps.as_ref());

        let res = modules_fn(&app, deps.as_ref());

        assert_that!(res)
            .is_err()
            .matches(|e| e.to_string().contains(&fake_module.to_string()));
    }

    mod app_request {
        use crate::mock_module::MockModuleExecuteMsg;
        use core::app;

        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.apps(deps);
                    mods.request(FAKE_MODULE_ID, MockModuleExecuteMsg {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_app_request() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.apps(deps.as_ref());

            let res = mods.request(TEST_MODULE_ID, MockModuleExecuteMsg {});

            let expected_msg: app::ExecuteMsg<_, Empty> =
                app::ExecuteMsg::Module(MockModuleExecuteMsg {});

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
        use core::app;

        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.apps(deps);
                    mods.configure(
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

            let mods = app.apps(deps.as_ref());

            let res = mods.configure(
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

    mod query_app {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.apps(deps);
                    mods.query::<_, Empty>(FAKE_MODULE_ID, Empty {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_app_query() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.apps(deps.as_ref());

            let res = mods.query::<_, String>(TEST_MODULE_ID, Empty {});

            assert_that!(res)
                .is_ok()
                .is_equal_to(abstract_testing::prelude::TEST_MODULE_RESPONSE.to_string());
        }
    }
}
