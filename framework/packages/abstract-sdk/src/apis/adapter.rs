#![allow(dead_code)]

use abstract_std::{adapter::AdapterRequestMsg, objects::module::ModuleId};
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps};
use serde::{de::DeserializeOwned, Serialize};

use super::{AbstractApi, ApiIdentification};
use crate::{
    cw_helpers::ApiQuery, features::ModuleIdentification, AbstractSdkResult, ModuleInterface,
};

/// Interact with other modules on the Account.
pub trait AdapterInterface: ModuleInterface + ModuleIdentification {
    /**
        API for accessing Abstract Adapters installed on the account.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # use abstract_testing::prelude::*;
        # let deps = mock_dependencies();
        # let account = admin_account(deps.api);
        # let module = MockModule::new(deps.api, account);

        let adapters: Adapters<MockModule>  = module.adapters(deps.as_ref());
        ```
    */
    fn adapters<'a>(&'a self, deps: Deps<'a>) -> Adapters<Self> {
        Adapters { base: self, deps }
    }
}

impl<T> AdapterInterface for T where T: ModuleInterface + ModuleIdentification {}

impl<'a, T: AdapterInterface> AbstractApi<T> for Adapters<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

impl<'a, T: AdapterInterface> ApiIdentification for Adapters<'a, T> {
    fn api_id() -> String {
        "Adapters".to_owned()
    }
}

/**
    API for accessing Abstract Adapters installed on the account.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # use abstract_testing::prelude::*;
    # let deps = mock_dependencies();
    # let account = admin_account(deps.api);
    # let module = MockModule::new(deps.api, account);

    let adapters: Adapters<MockModule>  = module.adapters(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Adapters<'a, T: AdapterInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: AdapterInterface> Adapters<'a, T> {
    /// Interactions with Abstract Adapters
    /// Construct an adapter execute message.
    pub fn execute<M: Serialize + Into<abstract_std::adapter::ExecuteMsg<M>>>(
        &self,
        adapter_id: ModuleId,
        message: M,
    ) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);
        modules.assert_module_dependency(adapter_id)?;
        let adapter_msg = abstract_std::adapter::ExecuteMsg::<_>::Module(AdapterRequestMsg::new(
            Some(self.base.account(self.deps)?.into_addr().into_string()),
            message,
        ));
        let adapter_address = modules.module_address(adapter_id)?;
        Ok(wasm_execute(adapter_address, &adapter_msg, vec![])?.into())
    }

    /// Smart query an Adapter
    pub fn query<Q: Serialize, R: DeserializeOwned>(
        &self,
        adapter_id: ModuleId,
        query: impl Into<abstract_std::adapter::QueryMsg<Q>>,
    ) -> AbstractSdkResult<R> {
        let adapter_query: abstract_std::adapter::QueryMsg<Q> = query.into();
        let modules = self.base.modules(self.deps);
        let adapter_address = modules.module_address(adapter_id)?;
        self.smart_query(adapter_address, &adapter_query)
    }
}

#[cfg(test)]
mod tests {

    use abstract_testing::prelude::*;
    use assertor::*;
    use cosmwasm_std::*;

    use super::*;
    use crate::mock_module::*;

    pub fn fail_when_not_dependency_test<T: std::fmt::Debug>(
        modules_fn: impl FnOnce(&MockModule, Deps) -> AbstractSdkResult<T>,
        fake_module: ModuleId,
    ) {
        let (deps, _, app) = mock_module_setup();

        let _mods = app.adapters(deps.as_ref());

        let res = modules_fn(&app, deps.as_ref());

        assert_that!(res)
            .err()
            .as_string()
            .contains(&fake_module.to_string());
    }
    mod adapter_request {
        use super::*;

        use crate::std::adapter;

        #[test]
        fn should_return_err_if_not_dependency() {
            fail_when_not_dependency_test(
                |app, deps| {
                    let mods = app.adapters(deps);
                    mods.execute(FAKE_MODULE_ID, MockModuleExecuteMsg {})
                },
                FAKE_MODULE_ID,
            );
        }

        #[test]
        fn expected_adapter_request() {
            let (deps, account, app) = mock_module_setup();
            let abstr = AbstractMockAddrs::new(deps.api);

            let mods = app.adapters(deps.as_ref());

            let res = mods.execute(TEST_MODULE_ID, MockModuleExecuteMsg {});

            let expected_msg: adapter::ExecuteMsg<_> =
                adapter::ExecuteMsg::Module(AdapterRequestMsg {
                    account_address: Some(account.addr().to_string()),
                    request: MockModuleExecuteMsg {},
                });

            assert_that!(res)
                .ok()
                .is_equal_to(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: abstr.module_address.to_string(),
                    msg: to_json_binary(&expected_msg).unwrap(),
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
            let (deps, _, app) = mock_module_setup();

            let mods = app.adapters(deps.as_ref());

            let inner_msg = Empty {};

            let res = mods.query::<_, String>(TEST_MODULE_ID, inner_msg);

            assert_that!(res)
                .ok()
                .is_equal_to(TEST_MODULE_RESPONSE.to_string());
        }
    }
}
