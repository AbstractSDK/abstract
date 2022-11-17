//! # Executor
//! The executor provides function for executing commands on the OS.
//!
use abstract_os::proxy::ExecuteMsg;
use cosmwasm_std::{
    to_binary, CosmosMsg, Deps, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};

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
    pub fn execute(&self, msgs: Vec<CosmosMsg>) -> Result<CosmosMsg, StdError> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.base.proxy_address(self.deps)?.to_string(),
            msg: to_binary(&ExecuteMsg::ModuleAction { msgs })?,
            funds: vec![],
        }))
    }
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
    pub fn execute_response(&self, msgs: Vec<CosmosMsg>, action: &str) -> StdResult<Response> {
        let msg = self.execute(msgs)?;
        Ok(Response::new()
            .add_message(msg)
            .add_attribute("action", action))
    }
}
