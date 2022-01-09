use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use dao_os::manager::queries::{query_module_addresses, query_module_versions};

use dao_os::manager::msg::{ModuleQueryResponse, VersionsQueryResponse};

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
