//! # Executor
//! The executor provides function for executing commands on the Account.
//!

use crate::{
    features::{AccountIdentification, ModuleIdentification},
    AbstractSdkResult, AccountAction,
};
use abstract_core::proxy::ExecuteMsg;
use abstract_macros::with_abstract_event;
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, ReplyOn, Response, SubMsg};

/// Execute an `AccountAction` on the Account.
pub trait Execution: AccountIdentification + ModuleIdentification {
    /**
        API for executing [`AccountAction`]s on the Account.
        Group your actions together in a single execute call if possible.

        Executing [`CosmosMsg`] on the account is possible by creating an [`AccountAction`].

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let executor: Executor<MockModule>  = module.executor(deps.as_ref());
        ```
    */
    fn executor<'a>(&'a self, deps: Deps<'a>) -> Executor<Self> {
        Executor { base: self, deps }
    }
}

impl<T> Execution for T where T: AccountIdentification + ModuleIdentification {}

/**
    API for executing [`AccountAction`]s on the Account.
    Group your actions together in a single execute call if possible.

    Executing [`CosmosMsg`] on the account is possible by creating an [`AccountAction`].

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let executor: Executor<MockModule>  = module.executor(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Executor<'a, T: Execution> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: Execution> Executor<'a, T> {
    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    pub fn execute(&self, actions: Vec<AccountAction>) -> AbstractSdkResult<CosmosMsg> {
        let msgs = actions.into_iter().flat_map(|a| a.messages()).collect();
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::ModuleAction { msgs },
            vec![],
        )?
        .into())
    }

    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    /// The execution will be executed in a submessage and the reply will be sent to the provided `reply_on`.
    pub fn execute_with_reply(
        &self,
        actions: Vec<AccountAction>,
        reply_on: ReplyOn,
        id: u64,
    ) -> AbstractSdkResult<SubMsg> {
        let msg = self.execute(actions)?;
        let sub_msg = SubMsg {
            id,
            msg,
            gas_limit: None,
            reply_on,
        };
        Ok(sub_msg)
    }

    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    /// Return a "standard" response for the executed messages. (with the provided action).
    pub fn execute_with_response(
        &self,
        actions: Vec<AccountAction>,
        action: &str,
    ) -> AbstractSdkResult<Response> {
        let msg = self.execute(actions)?;
        let resp = Response::default();

        Ok(with_abstract_event!(resp, self.base.module_id(), action).add_message(msg))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use abstract_core::proxy::ExecuteMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    fn mock_bank_send(amount: Vec<Coin>) -> AccountAction {
        AccountAction::from(vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: "to_address".to_string(),
            amount,
        })])
    }

    fn flatten_actions(actions: Vec<AccountAction>) -> Vec<CosmosMsg> {
        actions.into_iter().flat_map(|a| a.messages()).collect()
    }

    mod execute {
        use super::*;
        use cosmwasm_std::to_binary;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_actions() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            let messages = vec![];

            let actual_res = executor.execute(messages.clone().into());
            assert_that!(actual_res).is_ok();

            let expected = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_binary(&ExecuteMsg::ModuleAction {
                    msgs: flatten_actions(messages),
                })
                .unwrap(),
                funds: vec![],
            });
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }

        #[test]
        fn with_actions() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            // build a bank message
            let messages = vec![mock_bank_send(coins(100, "juno"))];

            let actual_res = executor.execute(messages.clone());
            assert_that!(actual_res).is_ok();

            let expected = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_binary(&ExecuteMsg::ModuleAction {
                    msgs: flatten_actions(messages),
                })
                .unwrap(),
                // funds should be empty
                funds: vec![],
            });
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }
    }

    mod execute_with_reply {
        use super::*;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_actions() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            let empty_actions = vec![];
            let expected_reply_on = ReplyOn::Success;
            let expected_reply_id = 10952;

            let actual_res = executor.execute_with_reply(
                empty_actions.clone(),
                expected_reply_on.clone(),
                expected_reply_id,
            );
            assert_that!(actual_res).is_ok();

            let expected = SubMsg {
                id: expected_reply_id,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_PROXY.to_string(),
                    msg: to_binary(&ExecuteMsg::ModuleAction {
                        msgs: flatten_actions(empty_actions),
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: expected_reply_on,
            };
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }

        #[test]
        fn with_actions() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            // build a bank message
            let action = vec![mock_bank_send(coins(1, "denom"))];
            // reply on never
            let expected_reply_on = ReplyOn::Never;
            let expected_reply_id = 1;

            let actual_res = executor.execute_with_reply(
                action.clone(),
                expected_reply_on.clone(),
                expected_reply_id,
            );
            assert_that!(actual_res).is_ok();

            let expected = SubMsg {
                id: expected_reply_id,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_PROXY.to_string(),
                    msg: to_binary(&ExecuteMsg::ModuleAction {
                        msgs: flatten_actions(action),
                    })
                    .unwrap(),
                    // funds should be empty
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: expected_reply_on,
            };
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }
    }

    mod execute_with_response {
        use super::*;
        use cosmwasm_std::coins;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_actions() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            let empty_actions = vec![];
            let expected_action = "THIS IS AN ACTION";

            let actual_res = executor.execute_with_response(empty_actions.clone(), expected_action);

            let expected_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_binary(&ExecuteMsg::ModuleAction {
                    msgs: flatten_actions(empty_actions),
                })
                .unwrap(),
                funds: vec![],
            });

            let expected = Response::new()
                .add_event(
                    Event::new("abstract")
                        .add_attribute("contract", stub.module_id())
                        .add_attribute("action", expected_action),
                )
                .add_message(expected_msg);

            assert_that!(actual_res).is_ok().is_equal_to(expected);
        }

        #[test]
        fn with_actions() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            // build a bank message
            let action = vec![mock_bank_send(coins(1, "denom"))];
            let expected_action = "provide liquidity";

            let actual_res = executor.execute_with_response(action.clone(), expected_action);

            let expected_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_binary(&ExecuteMsg::ModuleAction {
                    msgs: flatten_actions(action),
                })
                .unwrap(),
                // funds should be empty
                funds: vec![],
            });
            let expected = Response::new()
                .add_event(
                    Event::new("abstract")
                        .add_attribute("contract", stub.module_id())
                        .add_attribute("action", expected_action),
                )
                .add_message(expected_msg);
            assert_that!(actual_res).is_ok().is_equal_to(expected);
        }
    }
}
