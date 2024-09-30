//! # Executor
//! The executor provides function for executing commands on the Account.
//!

use abstract_macros::with_abstract_event;
use abstract_std::account::ExecuteMsg;
use cosmwasm_std::{Binary, Coin, CosmosMsg, Deps, ReplyOn, Response, SubMsg};

use super::{AbstractApi, ApiIdentification};
use crate::{
    features::{AccountExecutor, ModuleIdentification},
    AbstractSdkResult, AccountAction,
};

/// Execute an `AccountAction` on the Account.
pub trait Execution: AccountExecutor + ModuleIdentification {
    /**
        API for executing [`AccountAction`]s on the Account.
        Group your actions together in a single execute call if possible.

        Executing [`CosmosMsg`] on the account is possible by creating an [`AccountAction`].

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # use abstract_testing::prelude::*;
        # let deps = mock_dependencies();
        # let account = admin_account(deps.api);
        # let module = MockModule::new(deps.api, account);

        let executor: Executor<MockModule>  = module.executor(deps.as_ref());
        ```
    */
    fn executor<'a>(&'a self, deps: Deps<'a>) -> Executor<Self> {
        Executor { base: self, deps }
    }
}

impl<T> Execution for T where T: AccountExecutor + ModuleIdentification {}

impl<'a, T: Execution> AbstractApi<T> for Executor<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

impl<'a, T: Execution> ApiIdentification for Executor<'a, T> {
    fn api_id() -> String {
        "Executor".to_owned()
    }
}

/**
    API for executing [`AccountAction`]s on the Account.
    Group your actions together in a single execute call if possible.

    Executing [`CosmosMsg`] on the account is possible by creating an [`AccountAction`].

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # use abstract_testing::prelude::*;
    # let deps = mock_dependencies();
    # let account = admin_account(deps.api);
    # let module = MockModule::new(deps.api, account);

    let executor: Executor<MockModule>  = module.executor(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Executor<'a, T: Execution> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: Execution> Executor<'a, T> {
    /// Execute a single message on the `ModuleActionWithData` endpoint.
    fn execute_with_data(&self, msg: CosmosMsg) -> AbstractSdkResult<ExecutorMsg> {
        let msg = self.base.execute_on_account(
            self.deps,
            &ExecuteMsg::ModuleActionWithData { msg },
            vec![],
        )?;
        Ok(ExecutorMsg(msg))
    }

    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    pub fn execute(
        &self,
        actions: impl IntoIterator<Item = impl Into<AccountAction>>,
    ) -> AbstractSdkResult<ExecutorMsg> {
        self.execute_with_funds(actions, vec![])
    }

    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    /// Funds attached from sending module to proxy
    pub fn execute_with_funds(
        &self,
        actions: impl IntoIterator<Item = impl Into<AccountAction>>,
        funds: Vec<Coin>,
    ) -> AbstractSdkResult<ExecutorMsg> {
        let msgs = actions
            .into_iter()
            .flat_map(|a| a.into().messages())
            .collect();
        let msg =
            self.base
                .execute_on_account(self.deps, &ExecuteMsg::ModuleAction { msgs }, funds)?;
        Ok(ExecutorMsg(msg))
    }

    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    /// The execution will be executed in a submessage and the reply will be sent to the provided `reply_on`.
    pub fn execute_with_reply(
        &self,
        actions: impl IntoIterator<Item = impl Into<AccountAction>>,
        reply_on: ReplyOn,
        id: u64,
    ) -> AbstractSdkResult<SubMsg> {
        let msg = self.execute(actions)?;
        let sub_msg = SubMsg {
            id,
            msg: msg.into(),
            gas_limit: None,
            reply_on,
            payload: Binary::default(),
        };
        Ok(sub_msg)
    }

    /// Execute a single msg on the Account.
    /// This message will be executed on the proxy contract. Any data returned from the execution will be forwarded to the proxy's response through a reply.
    /// The resulting data should be available in the reply of the specified ID.
    pub fn execute_with_reply_and_data(
        &self,
        actions: CosmosMsg,
        reply_on: ReplyOn,
        id: u64,
    ) -> AbstractSdkResult<SubMsg> {
        let msg = self.execute_with_data(actions)?;
        let sub_msg = SubMsg {
            id,
            msg: msg.into(),
            gas_limit: None,
            reply_on,
            payload: Binary::default(),
        };
        Ok(sub_msg)
    }

    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    /// Return a "standard" response for the executed messages. (with the provided action).
    pub fn execute_with_response(
        &self,
        actions: impl IntoIterator<Item = impl Into<AccountAction>>,
        action: &str,
    ) -> AbstractSdkResult<Response> {
        let msg = self.execute(actions)?;
        let resp = Response::default();

        Ok(with_abstract_event!(resp, self.base.module_id(), action).add_message(msg))
    }
}

/// CosmosMsg from the executor methods
#[must_use = "ExecutorMsg should be provided to Response::add_message"]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Eq))]
pub struct ExecutorMsg(CosmosMsg);

impl From<ExecutorMsg> for CosmosMsg {
    fn from(val: ExecutorMsg) -> Self {
        val.0
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_std::account::ExecuteMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::*;
    use speculoos::prelude::*;

    use super::*;
    use crate::mock_module::*;

    fn mock_bank_send(amount: Vec<Coin>) -> AccountAction {
        AccountAction::from(CosmosMsg::Bank(BankMsg::Send {
            to_address: "to_address".to_string(),
            amount,
        }))
    }

    fn flatten_actions(actions: Vec<AccountAction>) -> Vec<CosmosMsg> {
        actions.into_iter().flat_map(|a| a.messages()).collect()
    }

    mod execute {
        use super::*;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_actions() {
            let (deps, account, stub) = mock_module_setup();
            let executor = stub.executor(deps.as_ref());

            let messages = vec![];

            let actual_res = executor.execute(messages.clone());
            assert_that!(actual_res).is_ok();

            let expected = ExecutorMsg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.addr().to_string(),
                msg: to_json_binary(&ExecuteMsg::ModuleAction::<cosmwasm_std::Empty> {
                    msgs: flatten_actions(messages),
                })
                .unwrap(),
                funds: vec![],
            }));
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }

        #[test]
        fn with_actions() {
            let (deps, account, stub) = mock_module_setup();
            let executor = stub.executor(deps.as_ref());

            // build a bank message
            let messages = vec![mock_bank_send(coins(100, "juno"))];

            let actual_res = executor.execute(messages.clone());
            assert_that!(actual_res).is_ok();

            let expected = ExecutorMsg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.addr().to_string(),
                msg: to_json_binary(&ExecuteMsg::ModuleAction::<cosmwasm_std::Empty> {
                    msgs: flatten_actions(messages),
                })
                .unwrap(),
                // funds should be empty
                funds: vec![],
            }));
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }
    }

    mod execute_with_reply {

        use super::*;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_actions() {
            let (deps, account, stub) = mock_module_setup();
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
                    contract_addr: account.addr().to_string(),
                    msg: to_json_binary(&ExecuteMsg::ModuleAction::<cosmwasm_std::Empty> {
                        msgs: flatten_actions(empty_actions),
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: expected_reply_on,
                payload: Binary::default(),
            };
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }

        #[test]
        fn with_actions() {
            let (deps, account, stub) = mock_module_setup();
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
                    contract_addr: account.addr().to_string(),
                    msg: to_json_binary(&ExecuteMsg::ModuleAction::<cosmwasm_std::Empty> {
                        msgs: flatten_actions(action),
                    })
                    .unwrap(),
                    // funds should be empty
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: expected_reply_on,
                payload: Binary::default(),
            };
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }
    }

    mod execute_with_response {
        use super::*;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_actions() {
            let (deps, account, stub) = mock_module_setup();
            let executor = stub.executor(deps.as_ref());

            let empty_actions = vec![];
            let expected_action = "THIS IS AN ACTION";

            let actual_res = executor.execute_with_response(empty_actions.clone(), expected_action);

            let expected_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.addr().to_string(),
                msg: to_json_binary(&ExecuteMsg::ModuleAction::<cosmwasm_std::Empty> {
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
            let (deps, account, stub) = mock_module_setup();

            let executor = stub.executor(deps.as_ref());

            // build a bank message
            let action = vec![mock_bank_send(coins(1, "denom"))];
            let expected_action = "provide liquidity";

            let actual_res = executor.execute_with_response(action.clone(), expected_action);

            let expected_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: account.addr().to_string(),
                msg: to_json_binary(&ExecuteMsg::ModuleAction::<cosmwasm_std::Empty> {
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
            assert_eq!(actual_res, Ok(expected));
        }
    }
}
