#![allow(dead_code)]

use crate::{AbstractSdkResult, ModuleInterface};
use abstract_core::{api::ApiRequestMsg, objects::module::ModuleId};
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, Empty};
use serde::{de::DeserializeOwned, Serialize};

/// Interact with other modules on the Account.
pub trait ApiInterface: ModuleInterface {
    fn apis<'a>(&'a self, deps: Deps<'a>) -> Api<Self> {
        Api { base: self, deps }
    }
}

impl<T> ApiInterface for T where T: ModuleInterface {}

#[derive(Clone)]
pub struct Api<'a, T: ApiInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ApiInterface> Api<'a, T> {
    /// Interactions with Abstract APIs
    /// Construct an api request message.
    pub fn request<M: Serialize + Into<abstract_core::api::ExecuteMsg<M, Empty>>>(
        &self,
        api_id: ModuleId,
        message: M,
    ) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);
        modules.assert_module_dependency(api_id)?;
        let api_msg = abstract_core::api::ExecuteMsg::<_>::Module(ApiRequestMsg::new(
            Some(self.base.proxy_address(self.deps)?.into_string()),
            message,
        ));
        let api_address = modules.module_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Smart query an API
    pub fn query<Q: Serialize, R: DeserializeOwned>(
        &self,
        api_id: ModuleId,
        message: impl Into<abstract_core::api::QueryMsg<Q>>,
    ) -> AbstractSdkResult<R> {
        let api_msg: abstract_core::api::QueryMsg<Q> = message.into();
        let modules = self.base.modules(self.deps);
        let api_address = modules.module_address(api_id)?;
        self.deps
            .querier
            .query_wasm_smart(api_address, &api_msg)
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

        let _mods = app.apis(deps.as_ref());

        let res = modules_fn(&app, deps.as_ref());

        assert_that!(res)
            .is_err()
            .matches(|e| e.to_string().contains(&fake_module.to_string()));
    }
    mod api_request {
        use super::*;
        use core::api::{self, ApiRequestMsg};

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.apis(deps);
                    mods.request(FAKE_MODULE_ID, MockModuleExecuteMsg {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_api_request() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.apis(deps.as_ref());

            let res = mods.request(TEST_MODULE_ID, MockModuleExecuteMsg {});

            let expected_msg: api::ExecuteMsg<_, Empty> = api::ExecuteMsg::Module(ApiRequestMsg {
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
                    let mods = app.apis(deps);
                    mods.query::<_, Empty>(FAKE_MODULE_ID, Empty {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_api_query() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let app = MockModule::new();

            let mods = app.apis(deps.as_ref());

            let inner_msg = Empty {};

            let res = mods.query::<_, String>(TEST_MODULE_ID, inner_msg);

            assert_that!(res)
                .is_ok()
                .is_equal_to(abstract_testing::prelude::TEST_MODULE_RESPONSE.to_string());
        }
    }
}
