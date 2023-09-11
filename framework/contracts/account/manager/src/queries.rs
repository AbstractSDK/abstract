use abstract_core::manager::state::{
    Config, ACCOUNT_MODULE_VERSIONS, SUB_ACCOUNTS, SUSPENSION_STATUS,
};
use abstract_core::manager::{AbstractContractVersion, SubAccountIdsResponse};
use abstract_sdk::core::manager::state::{AccountInfo, ACCOUNT_ID, ACCOUNT_MODULES, CONFIG, INFO};
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
    let vector = contracts.into_iter().map(|(v, k)| (v, k)).collect();
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
    let Config {
        version_control_address,
        module_factory_address,
        ..
    } = CONFIG.load(deps.storage)?;
    let is_suspended = SUSPENSION_STATUS.load(deps.storage)?;
    to_binary(&ConfigResponse {
        account_id,
        is_suspended,
        version_control_address,
        module_factory_address,
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
        let version = query_module_version(&deps, address.clone(), &id)?;
        resp_vec.push(ManagerModuleInfo {
            id,
            version,
            address,
        })
    }

    to_binary(&ModuleInfosResponse {
        module_infos: resp_vec,
    })
}

pub fn handle_sub_accounts_query(
    deps: Deps,
    last_account_id: Option<u32>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_account_id.map(Bound::exclusive);

    let res = SUB_ACCOUNTS
        .keys(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<u32>>>()?;

    to_binary(&SubAccountIdsResponse { sub_accounts: res })
}

/// RawQuery the version of an enabled module
pub fn query_module_version(
    deps: &Deps,
    module_addr: Addr,
    module_id: &str,
) -> StdResult<AbstractContractVersion> {
    let req = QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: module_addr.into(),
        key: CONTRACT.as_slice().into(),
    });
    match deps.querier.query::<ContractVersion>(&req) {
        Ok(v) => Ok(v.into()),
        Err(e) => {
            if let Some(version) = ACCOUNT_MODULE_VERSIONS.may_load(deps.storage, module_id)? {
                Ok(version.into())
            } else {
                Err(e)
            }
        }
    }
}

/// RawQuery the module versions of the modules part of the Account
/// Errors if not present
pub fn query_module_versions(
    deps: Deps,
    manager_addr: &Addr,
    module_names: &[String],
) -> StdResult<BTreeMap<String, AbstractContractVersion>> {
    let addresses: BTreeMap<String, Addr> =
        query_module_addresses(deps, manager_addr, module_names)?;
    let mut module_versions: BTreeMap<String, AbstractContractVersion> = BTreeMap::new();
    for (name, address) in addresses.into_iter() {
        let result = query_module_version(&deps, address, &name)?;
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
