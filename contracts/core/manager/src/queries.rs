use abstract_os::manager::state::{OsInfo, CONFIG, INFO, OS_ID, OS_MODULES, ROOT};
use abstract_os::manager::{
    ConfigResponse, InfoResponse, ManagerModuleInfo, ModuleAddressesResponse, ModuleInfosResponse,
    ModuleVersionsResponse,
};
use abstract_sdk::manager::{query_module_addresses, query_module_version, query_module_versions};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult, Uint64};
use cw_storage_plus::Bound;

const DEFAULT_LIMIT: u8 = 5;
const MAX_LIMIT: u8 = 10;

pub fn handle_module_address_query(deps: Deps, env: Env, ids: Vec<String>) -> StdResult<Binary> {
    let contracts = query_module_addresses(deps, &env.contract.address, &ids)?;
    let vector = contracts
        .into_iter()
        .map(|(v, k)| (v, k.to_string()))
        .collect();
    to_binary(&ModuleAddressesResponse { modules: vector })
}

pub fn handle_contract_versions_query(deps: Deps, env: Env, ids: Vec<String>) -> StdResult<Binary> {
    let response = query_module_versions(deps, &env.contract.address, &ids)?;
    let versions = response.into_values().collect();
    to_binary(&ModuleVersionsResponse { versions })
}

pub fn handle_os_info_query(deps: Deps) -> StdResult<Binary> {
    let info: OsInfo = INFO.load(deps.storage)?;
    to_binary(&InfoResponse { info })
}

pub fn handle_config_query(deps: Deps) -> StdResult<Binary> {
    let os_id = Uint64::from(OS_ID.load(deps.storage)?);
    let root = ROOT
        .get(deps)?
        .unwrap_or_else(|| Addr::unchecked(""))
        .to_string();
    let config = CONFIG.load(deps.storage)?;
    to_binary(&ConfigResponse {
        root,
        os_id,
        version_control_address: config.version_control_address.to_string(),
        module_factory_address: config.module_factory_address.into_string(),
    })
}
pub fn handle_module_info_query(
    deps: Deps,
    last_module_id: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_module_id.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(String, Addr)>, _> = OS_MODULES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    let ids_and_addr = res?;
    let mut resp_vec: Vec<ManagerModuleInfo> = vec![];
    for (id, address) in ids_and_addr.into_iter() {
        let version = query_module_version(&deps, address.clone())?;
        resp_vec.push(ManagerModuleInfo {
            id,
            version,
            address: address.to_string(),
        })
    }

    to_binary(&ModuleInfosResponse {
        module_infos: resp_vec,
    })
}
