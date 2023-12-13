#![allow(unused)]
use super::{AbstractApi, ApiIdentification};
use crate::{
    cw_helpers::ApiQuery,
    features::{DepsAccess, ModuleIdentification},
    AbstractSdkResult, AccountAction, ModuleInterface,
};
use abstract_core::objects::module::ModuleId;
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, Empty};
use serde::{de::DeserializeOwned, Serialize};

use abstract_core::app as msg;

/// Interact with other modules on the Account.
pub trait AppInterface: ModuleInterface + ModuleIdentification + DepsAccess {
    /**
        API for accessing Abstract Apps installed on the account.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let apps: Apps<MockModule>  = module.apps(deps.as_ref());
        ```
    */
    fn apps(&self) -> Apps<Self> {
        Apps { base: self }
    }
}

impl<T> AppInterface for T where T: ModuleInterface + ModuleIdentification + DepsAccess {}

impl<'a, T: AppInterface> AbstractApi<T> for Apps<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
}

impl<'a, T: AppInterface> ApiIdentification for Apps<'a, T> {
    fn api_id() -> String {
        "Apps".to_owned()
    }
}

/**
    API for accessing Abstract Apps installed on the account.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let apps: Apps<MockModule>  = module.apps(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Apps<'a, T: AppInterface> {
    base: &'a T,
}

impl<'a, T: AppInterface> Apps<'a, T> {
    /// Construct an app request message.
    pub fn request<M: Serialize>(
        &'a self,
        app_id: ModuleId,
        message: impl Into<msg::ExecuteMsg<M, Empty>>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.modules();
        modules.assert_module_dependency(app_id)?;
        let app_msg: msg::ExecuteMsg<M, Empty> = message.into();
        let app_address = modules.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Construct an app configuation message
    pub fn configure(
        &'a self,
        app_id: ModuleId,
        message: msg::BaseExecuteMsg,
    ) -> AbstractSdkResult<CosmosMsg> {
        let base_msg: msg::ExecuteMsg<Empty, Empty> = message.into();
        let modules = self.base.modules();
        let app_address = modules.module_address(app_id)?;
        Ok(wasm_execute(app_address, &base_msg, vec![])?.into())
    }

    /// Smart query an app
    pub fn query<Q: Serialize, R: DeserializeOwned>(
        &'a self,
        app_id: ModuleId,
        query: impl Into<msg::QueryMsg<Q>>,
    ) -> AbstractSdkResult<R> {
        let modules = self.base.modules();
        let app_query: msg::QueryMsg<Q> = query.into();
        let app_address = modules.module_address(app_id)?;
        self.smart_query(app_address, &app_query)
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
        modules_fn: impl FnOnce(&MockModule) -> AbstractSdkResult<T>,
        fake_module: ModuleId,
    ) {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let app = MockModule::new((deps.as_ref(), mock_env()).into());

        let _mods = app.apps();

        let res = modules_fn(&app);

        assert_that!(res)
            .is_err()
            .matches(|e| e.to_string().contains(&fake_module.to_string()));
    }

    mod app_request {
        use crate::core::app;
        use crate::mock_module::MockModuleExecuteMsg;

        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app| {
                    let mods = app.apps();
                    mods.request(FAKE_MODULE_ID, MockModuleExecuteMsg {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_app_request() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new((deps.as_ref(), mock_env()).into());

            let mods = app.apps();

            let res = mods.request(TEST_MODULE_ID, MockModuleExecuteMsg {});

            let expected_msg: app::ExecuteMsg<_, Empty> =
                app::ExecuteMsg::Module(MockModuleExecuteMsg {});

            assert_that!(res)
                .is_ok()
                .is_equal_to(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_MODULE_ADDRESS.into(),
                    msg: to_json_binary(&expected_msg).unwrap(),
                    funds: vec![],
                }));
        }
    }

    mod app_configure {
        use crate::core::app;

        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app| {
                    let mods = app.apps();
                    mods.configure(
                        FAKE_MODULE_ID,
                        app::BaseExecuteMsg::UpdateConfig {
                            ans_host_address: None,
                            version_control_address: None,
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
            let app = MockModule::new((deps.as_ref(), mock_env()).into());

            let mods = app.apps();

            let res = mods.configure(
                TEST_MODULE_ID,
                app::BaseExecuteMsg::UpdateConfig {
                    ans_host_address: Some("new_ans_addr".to_string()),
                    version_control_address: Some("new_vc_addr".to_string()),
                },
            );

            let expected_msg: app::ExecuteMsg<Empty, Empty> =
                app::ExecuteMsg::Base(app::BaseExecuteMsg::UpdateConfig {
                    ans_host_address: Some("new_ans_addr".to_string()),
                    version_control_address: Some("new_vc_addr".to_string()),
                });

            assert_that!(res)
                .is_ok()
                .is_equal_to::<CosmosMsg>(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_MODULE_ADDRESS.into(),
                    msg: to_json_binary(&expected_msg).unwrap(),
                    funds: vec![],
                }));
        }
    }

    mod query_app {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app| {
                    let mods = app.apps();
                    mods.query::<_, Empty>(FAKE_MODULE_ID, Empty {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_app_query() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new((deps.as_ref(), mock_env()).into());

            let mods = app.apps();

            let res = mods.query::<_, String>(TEST_MODULE_ID, Empty {});

            assert_that!(res)
                .is_ok()
                .is_equal_to(TEST_MODULE_RESPONSE.to_string());
        }
    }
}
