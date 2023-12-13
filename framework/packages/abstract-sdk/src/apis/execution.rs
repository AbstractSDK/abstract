//! # Executor
//! The executor provides function for executing commands on the Account.
//!

use super::{AbstractApi, ApiIdentification};
use crate::{
    account_action::{ExecuteOptions, ReplyOptions},
    features::{AccountIdentification, Executable, ExecutionStack, ModuleIdentification},
    AbstractSdkResult, AccountAction,
};
use abstract_core::proxy::ExecuteMsg;
use abstract_macros::with_abstract_event;
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, ReplyOn, Response, SubMsg};

/// Execute an `AccountAction` on the Account.
pub trait Execution: AccountIdentification + ModuleIdentification + ExecutionStack {
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
    fn executor<'a>(&'a mut self, deps: Deps<'a>) -> Executor<Self> {
        Executor { base: self, deps }
    }
}

impl<T> Execution for T where T: AccountIdentification + ModuleIdentification + ExecutionStack {}

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
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let executor: Executor<MockModule>  = module.executor(deps.as_ref());
    ```
*/
pub struct Executor<'a, T: Execution> {
    base: &'a mut T,
    deps: Deps<'a>,
}

impl<'a, T: Execution> Executor<'a, T> {
    /// Execute a single message on the `ModuleActionWithData` endpoint.
    fn execute_with_data(&self, msg: CosmosMsg) -> AbstractSdkResult<ExecutorMsg> {
        let msg = wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::ModuleActionWithData { msg },
            vec![],
        )?
        .into();
        Ok(ExecutorMsg(msg))
    }

    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    pub fn execute(&mut self, actions: Vec<CosmosMsg>) -> AbstractSdkResult<()> {
        self.execute_with_options(actions, ExecuteOptions::default())
    }

    // pub fn execute(&self, actions: Vec<AccountAction>) -> AbstractSdkResult<ExecutorMsg> {
    //     let msgs = actions.into_iter().flat_map(|a| a.messages()).collect();
    //     let msg: CosmosMsg = wasm_execute(
    //         self.base.proxy_address(self.deps)?.to_string(),
    //         &ExecuteMsg::ModuleAction { msgs },
    //         vec![],
    //     )?
    //     .into();
    //     Ok(ExecutorMsg(msg))
    // }

    /// Execute the msgs on the Account.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    /// The execution will be executed in a submessage and the reply will be sent to the provided `reply_on`.
    pub fn execute_with_reply(
        &mut self,
        msgs: Vec<CosmosMsg>,
        reply_on: ReplyOn,
        id: u64,
    ) -> AbstractSdkResult<()> {
        self.execute_with_options(
            msgs,
            ExecuteOptions {
                reply: Some(ReplyOptions {
                    reply_on,
                    id,
                    with_data: false,
                }),
            },
        )
    }

    /// Execute a single msg on the Account.
    /// This message will be executed on the proxy contract. Any data returned from the execution will be forwarded to the proxy's response through a reply.
    /// The resulting data should be available in the reply of the specified ID.
    pub fn execute_with_reply_and_data(
        &mut self,
        action: CosmosMsg,
        reply_on: ReplyOn,
        id: u64,
    ) -> AbstractSdkResult<()> {
        self.execute_with_options(
            vec![action],
            ExecuteOptions {
                reply: Some(ReplyOptions {
                    reply_on,
                    id,
                    with_data: true,
                }),
            },
        )
    }

    /// Executes multiple messages with options on the underlying account
    /// The messages will be executed on the proxy contract.
    pub fn execute_with_options(
        &mut self,
        msgs: Vec<CosmosMsg>,
        options: ExecuteOptions,
    ) -> AbstractSdkResult<()> {
        self.base.push_executable(Executable::AccountAction(
            AccountAction::from_vec_with_options(msgs, options)?,
        ));
        Ok(())
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
    use super::*;
    use crate::mock_module::*;
    use abstract_core::proxy::ExecuteMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    fn mock_bank_send(amount: Vec<Coin>) -> CosmosMsg {
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "to_address".to_string(),
            amount,
        })
    }

    fn flatten_actions(actions: Vec<AccountAction>) -> Vec<CosmosMsg> {
        actions.into_iter().flat_map(|a| a.messages()).collect()
    }

    mod execute {
        use crate::features::ResponseGenerator;

        use super::*;
        use cosmwasm_std::to_json_binary;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_actions() {
            let deps = mock_dependencies();
            let mut stub = MockModule::new();
            let mut executor = stub.executor(deps.as_ref());

            let messages = vec![];

            let actual_res = executor.execute(messages.clone());
            assert_that!(actual_res).is_ok();

            let expected = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_json_binary(&ExecuteMsg::ModuleAction { msgs: messages }).unwrap(),
                funds: vec![],
            });
            assert_that!(stub._generate_response(deps.as_ref()).unwrap().messages[0])
                .is_equal_to(SubMsg::new(expected));
        }

        #[test]
        fn with_actions() {
            let deps = mock_dependencies();
            let mut stub = MockModule::new();
            let mut executor = stub.executor(deps.as_ref());

            // build a bank message
            let messages = vec![mock_bank_send(coins(100, "juno"))];

            let actual_res = executor.execute(messages.clone());
            assert_that!(actual_res).is_ok();

            let expected = ExecutorMsg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_json_binary(&ExecuteMsg::ModuleAction { msgs: messages }).unwrap(),
                // funds should be empty
                funds: vec![],
            }));

            assert_that!(stub._generate_response(deps.as_ref()).unwrap().messages[0])
                .is_equal_to(SubMsg::new(expected));
        }
    }

    mod execute_with_reply {
        use crate::features::ResponseGenerator;

        use super::*;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_actions() {
            let deps = mock_dependencies();
            let mut stub = MockModule::new();
            let mut executor = stub.executor(deps.as_ref());

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
                    msg: to_json_binary(&ExecuteMsg::ModuleAction {
                        msgs: empty_actions,
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: expected_reply_on,
            };

            assert_that!(stub._generate_response(deps.as_ref()).unwrap().messages[0])
                .is_equal_to(expected);
        }

        #[test]
        fn with_actions() {
            let deps = mock_dependencies();
            let mut stub = MockModule::new();
            let mut executor = stub.executor(deps.as_ref());

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
                    msg: to_json_binary(&ExecuteMsg::ModuleAction { msgs: action }).unwrap(),
                    // funds should be empty
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: expected_reply_on,
            };

            assert_that!(stub._generate_response(deps.as_ref()).unwrap().messages[0])
                .is_equal_to(expected);
        }
    }
}
