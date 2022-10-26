use abstract_os::objects::memory::Memory;
use cosmwasm_std::{Addr, Coin, CosmosMsg, Deps, Response, StdResult, Storage};
use serde::Serialize;

use crate::Resolve;

/// execute an operation on the os
pub trait OsExecute {
    type Err: ToString;

    fn os_execute(&self, deps: Deps, msgs: Vec<CosmosMsg>) -> Result<Response, Self::Err>;
    fn os_ibc_execute(
        &self,
        deps: Deps,
        msgs: Vec<abstract_os::ibc_client::ExecuteMsg>,
    ) -> Result<Response, Self::Err>;
}

/// easily retrieve the memory object from the contract to perform queries
pub trait MemoryOperation {
    /// Load the Memory object
    fn load_memory(&self, store: &dyn Storage) -> StdResult<Memory>;
    /// Resolve a query on the memory contract
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
