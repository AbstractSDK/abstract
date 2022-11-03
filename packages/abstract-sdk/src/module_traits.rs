use abstract_os::objects::memory::Memory;
use cosmwasm_std::{Addr, Coin, CosmosMsg, Deps, ReplyOn, StdError, StdResult, Storage, SubMsg};
use serde::Serialize;

use crate::Resolve;

/// Construct the call on the proxy contract of the OS
pub trait OsExecute {
    fn os_execute(&self, deps: Deps, msgs: Vec<CosmosMsg>) -> Result<SubMsg, StdError>;
    fn os_ibc_execute(
        &self,
        deps: Deps,
        msgs: Vec<abstract_os::ibc_client::ExecuteMsg>,
    ) -> Result<SubMsg, StdError>;
    fn os_execute_with_reply(
        &self,
        deps: Deps,
        msgs: Vec<CosmosMsg>,
        reply_on: ReplyOn,
        id: u64,
    ) -> Result<SubMsg, StdError> {
        let mut msg = self.os_execute(deps, msgs)?;
        msg.reply_on = reply_on;
        msg.id = id;
        Ok(msg)
    }
}

/// easily retrieve the memory object from the contract to perform queries
pub trait MemoryOperation {
    /// Load the Memory object
    fn load_memory(&self, store: &dyn Storage) -> StdResult<Memory>;
    /// Resolve a query on the memory contract
    /// Use if only 1-2 queries are required
    /// loads the Memory var every call
    fn resolve<T: Resolve>(&self, deps: Deps, memory_entry: &T) -> StdResult<T::Output> {
        memory_entry.resolve(deps, &self.load_memory(deps.storage)?)
    }
}

/// Call functions on dependencies
pub trait Dependency {
    fn dependency_address(&self, deps: Deps, dependency_name: &str) -> StdResult<Addr>;
    fn call_api_dependency<E: Serialize>(
        &self,
        deps: Deps,
        dependency_name: &str,
        request_msg: &E,
        funds: Vec<Coin>,
    ) -> StdResult<CosmosMsg>;
}
