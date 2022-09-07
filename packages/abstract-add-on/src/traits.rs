use abstract_os::api::ApiRequestMsg;
use abstract_sdk::{
    api_request, manager::query_module_address, proxy::send_to_proxy, Dependency, MemoryOperation,
    OsExecute,
};
use cosmwasm_std::{Addr, Deps, Response, StdError, StdResult, Storage};

use crate::{AddOnContract, AddOnError};

impl MemoryOperation for AddOnContract<'_> {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}

impl OsExecute for AddOnContract<'_> {
    type Err = AddOnError;
    fn os_execute(
        &self,
        deps: Deps,
        msgs: Vec<cosmwasm_std::CosmosMsg>,
    ) -> Result<Response, Self::Err> {
        let proxy = self.base_state.load(deps.storage)?.proxy_address;
        Ok(Response::new().add_message(send_to_proxy(msgs, &proxy)?))
    }
}

impl Dependency for AddOnContract<'_> {
    fn dependency_address(&self, deps: Deps, dependency_name: &str) -> StdResult<Addr> {
        let manager_addr = &self
            .admin
            .get(deps)?
            .ok_or_else(|| StdError::generic_err("No admin on add-on"))?;
        query_module_address(deps, manager_addr, dependency_name)
    }
    fn call_api_dependency<E: serde::Serialize>(
        &self,
        deps: Deps,
        dependency_name: &str,
        request_msg: &E,
        funds: Vec<cosmwasm_std::Coin>,
    ) -> StdResult<cosmwasm_std::CosmosMsg> {
        let dep_addr = self.dependency_address(deps, dependency_name)?;
        let proxy_addr = self.state(deps.storage)?.proxy_address;
        let api_request_msg = ApiRequestMsg {
            proxy_address: Some(proxy_addr.to_string()),
            request: request_msg,
        };
        api_request(dep_addr, api_request_msg, funds)
    }
}
