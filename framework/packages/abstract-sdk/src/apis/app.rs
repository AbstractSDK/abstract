#![allow(unused)]
use abstract_std::{app as msg, objects::module::ModuleId};
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, Empty};
use serde::{de::DeserializeOwned, Serialize};

use super::{AbstractApi, ApiIdentification};
use crate::{
    cw_helpers::ApiQuery, features::ModuleIdentification, AbstractSdkResult, AccountAction,
    ModuleInterface,
};

/// Interact with other modules on the Account.
pub trait AppInterface: ModuleInterface + ModuleIdentification {
    /**
        API for accessing Abstract Apps installed on the account.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let deps = mock_dependencies();
        # let module = MockModule::new(deps.api);

        let apps: Apps<MockModule>  = module.apps(deps.as_ref());
        ```
    */
    fn apps<'a>(&'a self, deps: Deps<'a>) -> Apps<Self> {
        Apps { base: self, deps }
    }
}

impl<T> AppInterface for T where T: ModuleInterface + ModuleIdentification {}

impl<'a, T: AppInterface> AbstractApi<T> for Apps<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
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
    # let deps = mock_dependencies();
    # let module = MockModule::new(deps.api);

    let apps: Apps<MockModule>  = module.apps(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Apps<'a, T: AppInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: AppInterface> Apps<'a, T> {
    /// Construct an app request message.
    pub fn execute<M: Serialize>(
        &self,
        app_id: ModuleId,
        message: impl Into<msg::ExecuteMsg<M>>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);
        modules.assert_module_dependency(app_id)?;
        let app_msg: msg::ExecuteMsg<M> = message.into();
        let app_address = modules.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Smart query an app
    pub fn query<Q: Serialize, R: DeserializeOwned>(
        &self,
        app_id: ModuleId,
        query: impl Into<msg::QueryMsg<Q>>,
    ) -> AbstractSdkResult<R> {
        let modules = self.base.modules(self.deps);
        let app_query: msg::QueryMsg<Q> = query.into();
        let app_address = modules.module_address(app_id)?;
        self.smart_query(app_address, &app_query)
    }
}

#[cfg(test)]
mod tests {
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    pub use super::*;
    use crate::mock_module::*;
    /// Helper to check that the method is not callable when the module is not a dependency
    fn fail_when_not_dependency_test<T: std::fmt::Debug>(
        modules_fn: impl FnOnce(&MockModule, Deps) -> AbstractSdkResult<T>,
        fake_module: ModuleId,
    ) {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier(deps.api);
        let app = MockModule::new(deps.api);

        let _mods = app.apps(deps.as_ref());

        let res = modules_fn(&app, deps.as_ref());

        assert_that!(res)
            .is_err()
            .matches(|e| e.to_string().contains(&fake_module.to_string()));
    }

    mod app_request {
        use super::*;
        use crate::{mock_module::MockModuleExecuteMsg, std::app};

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.apps(deps);
                    mods.execute(FAKE_MODULE_ID, MockModuleExecuteMsg {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_app_request() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier(deps.api);
            let app = MockModule::new(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            let mods = app.apps(deps.as_ref());

            let res = mods.execute(TEST_MODULE_ID, MockModuleExecuteMsg {});

            let expected_msg: app::ExecuteMsg<_> = app::ExecuteMsg::Module(MockModuleExecuteMsg {});

            assert_that!(res)
                .is_ok()
                .is_equal_to(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: abstr.module_address.into(),
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
            deps.querier = abstract_testing::mock_querier(deps.api);
            let app = MockModule::new(deps.api);

            let mods = app.apps(deps.as_ref());

            let res = mods.query::<_, String>(TEST_MODULE_ID, Empty {});

            assert_that!(res)
                .is_ok()
                .is_equal_to(TEST_MODULE_RESPONSE.to_string());
        }
    }
}
