//! # Executor
//! The executor provides function for executing commands on the OS.
//!
use abstract_os::proxy::ExecuteMsg;
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, ReplyOn, Response, StdError, StdResult, SubMsg};

use super::Identification;

/// Execute an arbitrary `CosmosMsg` action on the OS.
pub trait Execution: Identification {
    fn executor<'a>(&'a self, deps: Deps<'a>) -> Executor<Self> {
        Executor { base: self, deps }
    }
}

impl<T> Execution for T where T: Identification {}

#[derive(Clone)]
pub struct Executor<'a, T: Execution> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: Execution> Executor<'a, T> {
    /// Execute the msgs on the OS.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    pub fn execute(&self, msgs: Vec<CosmosMsg>) -> Result<CosmosMsg, StdError> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::ModuleAction { msgs },
            vec![],
        )?
        .into())
    }

    /// Execute the msgs on the OS.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    /// The execution will be executed in a submessage and the reply will be sent to the provided `reply_on`.
    pub fn execute_with_reply(
        &self,
        msgs: Vec<CosmosMsg>,
        reply_on: ReplyOn,
        id: u64,
    ) -> Result<SubMsg, StdError> {
        let msg = self.execute(msgs)?;
        let sub_msg = SubMsg {
            id,
            msg,
            gas_limit: None,
            reply_on,
        };
        Ok(sub_msg)
    }

    /// Execute the msgs on the OS.
    /// These messages will be executed on the proxy contract and the sending module must be whitelisted.
    /// Return a "standard" response for the executed messages. (with the provided action).
    pub fn execute_with_response(&self, msgs: Vec<CosmosMsg>, action: &str) -> StdResult<Response> {
        let msg = self.execute(msgs)?;
        Ok(Response::new()
            .add_message(msg)
            .add_attribute("action", action))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::apis::test_common::*;

    fn mock_bank_send(amount: Vec<Coin>) -> CosmosMsg {
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "to_address".to_string(),
            amount,
        })
    }

    mod execute {
        use super::*;
        use cosmwasm_std::to_binary;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_msgs() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            let messages = vec![];

            let actual_res = executor.execute(messages.clone());
            assert_that!(actual_res).is_ok();

            let expected = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_binary(&ExecuteMsg::ModuleAction { msgs: messages }).unwrap(),
                funds: vec![],
            });
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }

        #[test]
        fn with_msgs() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            // build a bank message
            let messages = vec![mock_bank_send(coins(100, "juno"))];

            let actual_res = executor.execute(messages.clone());
            assert_that!(actual_res).is_ok();

            let expected = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_binary(&ExecuteMsg::ModuleAction { msgs: messages }).unwrap(),
                // funds should be empty
                funds: vec![],
            });
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }
    }

    mod execute_with_reply {
        use super::*;
        use crate::apis::test_common::TEST_PROXY;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_msgs() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            let empty_msgs = vec![];
            let expected_reply_on = ReplyOn::Success;
            let expected_reply_id = 10952;

            let actual_res = executor.execute_with_reply(
                empty_msgs.clone(),
                expected_reply_on.clone(),
                expected_reply_id,
            );
            assert_that!(actual_res).is_ok();

            let expected = SubMsg {
                id: expected_reply_id,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_PROXY.to_string(),
                    msg: to_binary(&ExecuteMsg::ModuleAction { msgs: empty_msgs }).unwrap(),
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: expected_reply_on,
            };
            assert_that!(actual_res.unwrap()).is_equal_to(expected);
        }

        #[test]
        fn with_msgs() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            // build a bank message
            let messages = vec![mock_bank_send(coins(1, "denom"))];
            // reply on never
            let expected_reply_on = ReplyOn::Never;
            let expected_reply_id = 1;

            let actual_res = executor.execute_with_reply(
                messages.clone(),
                expected_reply_on.clone(),
                expected_reply_id,
            );
            assert_that!(actual_res).is_ok();

            let expected = SubMsg {
                id: expected_reply_id,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: TEST_PROXY.to_string(),
                    msg: to_binary(&ExecuteMsg::ModuleAction { msgs: messages }).unwrap(),
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
        use crate::apis::test_common::TEST_PROXY;
        use cosmwasm_std::coins;

        /// Tests that no error is thrown with empty messages provided
        #[test]
        fn empty_msgs() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            let empty_msgs = vec![];
            let expected_action = "THIS IS AN ACTION";

            let actual_res = executor.execute_with_response(empty_msgs.clone(), expected_action);

            let expected_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_binary(&ExecuteMsg::ModuleAction { msgs: empty_msgs }).unwrap(),
                funds: vec![],
            });

            let expected = Response::new()
                .add_message(expected_msg)
                .add_attribute("action", expected_action);

            assert_that!(actual_res).is_ok().is_equal_to(expected);
        }

        #[test]
        fn with_msgs() {
            let deps = mock_dependencies();
            let stub = MockModule::new();
            let executor = stub.executor(deps.as_ref());

            // build a bank message
            let messages = vec![mock_bank_send(coins(1, "denom"))];
            let expected_action = "provide liquidity";

            let actual_res = executor.execute_with_response(messages.clone(), expected_action);

            let expected_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_PROXY.to_string(),
                msg: to_binary(&ExecuteMsg::ModuleAction { msgs: messages }).unwrap(),
                // funds should be empty
                funds: vec![],
            });
            let expected = Response::new()
                .add_message(expected_msg)
                .add_attribute("action", expected_action);
            assert_that!(actual_res).is_ok().is_equal_to(expected);
        }
    }
}
