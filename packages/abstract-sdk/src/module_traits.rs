use abstract_os::objects::memory::Memory;
use cosmwasm_std::{CosmosMsg, Deps, Response, StdResult, Storage};

use crate::Resolve;

/// execute an operation on the os
pub trait OsExecute {
    type Err: ToString;

    fn os_execute(&self, deps: Deps, msgs: Vec<CosmosMsg>) -> Result<Response, Self::Err>;
}

// easily retrieve the memory object from the contract to perform queries
pub trait MemoryOperation {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<Memory>;
    fn resolve<T: Resolve>(&self, deps: Deps, memory_entry: &T) -> StdResult<T::Output> {
        memory_entry.resolve(deps, &self.load_memory(deps.storage)?)
    }
}
