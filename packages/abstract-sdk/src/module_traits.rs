use abstract_os::objects::memory::Memory;
use cosmwasm_std::{CosmosMsg, Deps, Response, StdResult, Storage};

/// execute an operation on the os
pub trait OsExecute {
    type Err: ToString;

    fn os_execute(&self, deps: Deps, msgs: Vec<CosmosMsg>) -> Result<Response, Self::Err>;
}

// easily retrieve the memory object from the contract to perform queries
pub trait LoadMemory {
    fn mem(&self, store: &dyn Storage) -> StdResult<Memory>;
}
