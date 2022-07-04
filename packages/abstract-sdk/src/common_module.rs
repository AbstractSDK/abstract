use cosmwasm_std::{CosmosMsg, Deps, Response, StdResult, Storage};

use abstract_os::objects::memory::Memory;

use cw_controllers::Admin;

pub const BASE_STATE_KEY: &str = "\u{0}{10}base_state";
pub const ADMIN_KEY: &str = "admin";
pub const ADMIN: Admin = Admin::new(ADMIN_KEY);

/// execute an operation on the proxy
pub trait ProxyExecute {
    type Err: ToString;

    fn execute_on_proxy(&self, deps: Deps, msgs: Vec<CosmosMsg>) -> Result<Response, Self::Err>;
}

// easily retrieve the memory object from the contract to perform queries
pub trait Mem {
    fn mem(&self, store: &dyn Storage) -> StdResult<Memory>;
}
