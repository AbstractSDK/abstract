use cosmwasm_std::QueryRequest;
use cosmwasm_std::WasmQuery;

use crate::state::OS_ADDRESSES;
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};
use cw_storage_plus::U32Key;

use dao_os::manager::msg::{EnabledModulesResponse, QueryMsg};

pub fn query_enabled_modules(deps: Deps, manager_addr: Addr) -> StdResult<Binary> {
    let response: EnabledModulesResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: manager_addr.to_string(),
            msg: to_binary(&QueryMsg::QueryEnabledModules {})?,
        }))?;
    to_binary(&response)
}

pub fn query_os_address(deps: Deps, os_id: u32) -> StdResult<Binary> {
    let address: String = OS_ADDRESSES.load(deps.storage, U32Key::new(os_id))?;
    to_binary(&address)
}
