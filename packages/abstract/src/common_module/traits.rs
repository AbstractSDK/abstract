use cosmwasm_std::{CosmosMsg, Deps, Response, StdResult, Storage};

use crate::native::memory::item::Memory;

/// execute an operation on the proxy
pub trait ProxyExecute {
    type Err: ToString;

    fn execute_on_proxy(&self, deps: Deps, msgs: Vec<CosmosMsg>) -> Result<Response, Self::Err>;
}

// easily retrieve the memory object from the contract to perform queries
pub trait Mem {
    fn mem(&self, store: &dyn Storage) -> StdResult<Memory>;
}
