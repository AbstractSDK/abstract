use cosmwasm_std::QueryRequest;
use cosmwasm_std::WasmQuery;
use cw_storage_plus::Prefix;
use cw_storage_plus::Bound;
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult, Order};
use manager::state::OS_MODULES;
use dao_os::manager::msg::{EnabledModulesResponse, QueryMsg};

pub fn query_enabled_modules(deps: Deps, manager_addr: Addr) -> StdResult<Binary> {
    let response: EnabledModulesResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: manager_addr.to_string(),
            msg: to_binary(&QueryMsg::QueryEnabledModules {})?,
        }))?;
    to_binary(&response)
}