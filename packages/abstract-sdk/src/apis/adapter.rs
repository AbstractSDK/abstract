#![allow(dead_code)]

use crate::{AbstractSdkResult, ModuleInterface};
use abstract_core::{adapter::AdapterRequestMsg, objects::module::ModuleId};
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, Empty};
use serde::{de::DeserializeOwned, Serialize};

/// Interact with other modules on the Account.
pub trait AdapterInterface: ModuleInterface {
    /**
        API for accessing Abstract Adapters installed on the account.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let adapter: Adapter<MockModule>  = module.adapters(deps.as_ref());
        ```
    */
    fn adapters<'a>(&'a self, deps: Deps<'a>) -> Adapters<Self> {
        Adapters { base: self, deps }
    }
}

impl<T> AdapterInterface for T where T: ModuleInterface {}

/**
    API for accessing Abstract Adapters installed on the account.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let adapter: Adapter<MockModule>  = module.adapters(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Adapters<'a, T: AdapterInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: AdapterInterface> Adapters<'a, T> {
    /// Interactions with Abstract Adapters
    /// Construct an adapter request message.
    pub fn request<M: Serialize + Into<abstract_core::adapter::ExecuteMsg<M, Empty>>>(
        &self,
        adapter_id: ModuleId,
        message: M,
    ) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);
        modules.assert_module_dependency(adapter_id)?;
        let adapter_msg = abstract_core::adapter::ExecuteMsg::<_>::Module(AdapterRequestMsg::new(
            Some(self.base.proxy_address(self.deps)?.into_string()),
            message,
        ));
        let adapter_address = modules.module_address(adapter_id)?;
        Ok(wasm_execute(adapter_address, &adapter_msg, vec![])?.into())
    }

    /// Smart query an Adapter
    pub fn query<Q: Serialize, R: DeserializeOwned>(
        &self,
        adapter_id: ModuleId,
        query: impl Into<abstract_core::adapter::QueryMsg<Q>>,
    ) -> AbstractSdkResult<R> {
        let adapter_query: abstract_core::adapter::QueryMsg<Q> = query.into();
        let modules = self.base.modules(self.deps);
        let adapter_address = modules.module_address(adapter_id)?;
        self.deps
            .querier
            .query_wasm_smart(adapter_address, &adapter_query)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {

    use crate::mock_module::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::{assert_that, result::ResultAssertions};

    use super::*;

    pub fn fail_when_not_dependency_test<T: std::fmt::Debug>(
        modules_fn: impl FnOnce(&MockModule, Deps) -> AbstractSdkResult<T>,
        fake_module: ModuleId,
    ) {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let app = MockModule::new();

        let _mods = app.adapters(deps.as_ref());

        let res = modules_fn(&app, deps.as_ref());

        assert_that!(res)
            .is_err()
            .matches(|e| e.to_string().contains(&fake_module.to_string()));
    }
    mod adapter_request {
        use super::*;
        use core::adapter::{self, AdapterRequestMsg};

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.adapters(deps);
                    mods.request(FAKE_MODULE_ID, MockModuleExecuteMsg {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_adapter_request() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.adapters(deps.as_ref());

            let res = mods.request(TEST_MODULE_ID, MockModuleExecuteMsg {});

            let expected_msg: adapter::ExecuteMsg<_, Empty> =
                adapter::ExecuteMsg::Module(AdapterRequestMsg {
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

    mod query_api {
        use super::*;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.adapters(deps);
                    mods.query::<_, Empty>(FAKE_MODULE_ID, Empty {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_adapter_query() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.adapters(deps.as_ref());

            let inner_msg = Empty {};

            let res = mods.query::<_, String>(TEST_MODULE_ID, inner_msg);

            assert_that!(res)
                .is_ok()
                .is_equal_to(abstract_testing::prelude::TEST_MODULE_RESPONSE.to_string());
        }
    }
}
