use crate::state::OS_MODULES;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};
use pandora::manager::msg::{ModuleQueryResponse, VersionsQueryResponse};
use pandora::manager::{
    msg::EnabledModulesResponse,
    queries::{query_module_addresses, query_module_versions},
};

pub fn handle_module_addresses_query(
    deps: Deps,
    env: Env,
    names: Vec<String>,
) -> StdResult<Binary> {
    let contracts = query_module_addresses(deps, &env.contract.address, &names)?;
    let vector = contracts
        .into_iter()
        .map(|(v, k)| (v, k.to_string()))
        .collect();
    to_binary(&ModuleQueryResponse { modules: vector })
}

pub fn handle_contract_versions_query(
    deps: Deps,
    env: Env,
    names: Vec<String>,
) -> StdResult<Binary> {
    let response = query_module_versions(deps, &env.contract.address, &names)?;
    let versions = response.into_values().collect();
    to_binary(&VersionsQueryResponse { versions })
}

pub fn handle_enabled_modules_query(deps: Deps) -> StdResult<Binary> {
    let module_names: Vec<String> = OS_MODULES
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|module| String::from_utf8(module).unwrap())
        .collect();

    // for module in modules.into_iter() {
    //     module_names.push(String::from_utf8(module)?);
    // };

    to_binary(&EnabledModulesResponse {
        modules: module_names,
    })
}
