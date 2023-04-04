use abstract_core::manager::state::SUSPENSION_STATUS;
use abstract_sdk::core::manager::state::{
    AccountInfo, ACCOUNT_ID, ACCOUNT_MODULES, CONFIG, INFO, OWNER,
};
use abstract_sdk::core::manager::{
    ConfigResponse, InfoResponse, ManagerModuleInfo, ModuleAddressesResponse, ModuleInfosResponse,
    ModuleVersionsResponse,
};
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, Env, Order, QueryRequest, StdError, StdResult, Uint64, WasmQuery,
};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::Bound;
use std::collections::BTreeMap;

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

pub fn handle_account_info_query(deps: Deps) -> StdResult<Binary> {
    let info: AccountInfo = INFO.load(deps.storage)?;
    to_binary(&InfoResponse { info })
}

pub fn handle_config_query(deps: Deps) -> StdResult<Binary> {
    let account_id = Uint64::from(ACCOUNT_ID.load(deps.storage)?);
    let owner = OWNER
        .get(deps)?
        .unwrap_or_else(|| Addr::unchecked(""))
        .to_string();
    let config = CONFIG.load(deps.storage)?;
    let is_suspended = SUSPENSION_STATUS.load(deps.storage)?;
    to_binary(&ConfigResponse {
        owner,
        account_id,
        is_suspended,
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

    let res: Result<Vec<(String, Addr)>, _> = ACCOUNT_MODULES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    let ids_and_addr = res?;
    let mut resp_vec: Vec<ManagerModuleInfo> = vec![];
    for (id, address) in ids_and_addr.into_iter() {
        let version = query_module_cw2(&deps, address.clone())?;
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

/// RawQuery the version of an enabled module
pub fn query_module_cw2(deps: &Deps, module_addr: Addr) -> StdResult<ContractVersion> {
    let req = QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: module_addr.into(),
        key: CONTRACT.as_slice().into(),
    });
    deps.querier.query::<ContractVersion>(&req)
}

/// RawQuery the module versions of the modules part of the Account
/// Errors if not present
pub fn query_module_versions(
    deps: Deps,
    manager_addr: &Addr,
    module_names: &[String],
) -> StdResult<BTreeMap<String, ContractVersion>> {
    let addresses: BTreeMap<String, Addr> =
        query_module_addresses(deps, manager_addr, module_names)?;
    let mut module_versions: BTreeMap<String, ContractVersion> = BTreeMap::new();
    for (name, address) in addresses.into_iter() {
        let result = query_module_cw2(&deps, address)?;
        module_versions.insert(name, result);
    }
    Ok(module_versions)
}

/// RawQuery module addresses from manager
/// Errors if not present
pub fn query_module_addresses(
    deps: Deps,
    manager_addr: &Addr,
    module_names: &[String],
) -> StdResult<BTreeMap<String, Addr>> {
    let mut modules: BTreeMap<String, Addr> = BTreeMap::new();

    // Query over
    for module in module_names.iter() {
        let result: StdResult<Addr> = ACCOUNT_MODULES
            .query(&deps.querier, manager_addr.clone(), module)?
            .ok_or_else(|| {
                StdError::generic_err(format!("Module {module} not present in Account"))
            });
        // Add to map if present, skip otherwise. Allows version control to check what modules are present.
        match result {
            Ok(address) => modules.insert(module.clone(), address),
            Err(_) => None,
        };
    }
    Ok(modules)
}
