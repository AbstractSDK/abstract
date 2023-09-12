use abstract_core::manager::state::{Config, SUB_ACCOUNTS, SUSPENSION_STATUS};
use abstract_core::manager::SubAccountIdsResponse;
use abstract_core::objects::module::{self, ModuleInfo};
use abstract_sdk::core::manager::state::{AccountInfo, ACCOUNT_ID, ACCOUNT_MODULES, CONFIG, INFO};
use abstract_sdk::core::manager::{
    ConfigResponse, InfoResponse, ManagerModuleInfo, ModuleAddressesResponse, ModuleInfosResponse,
    ModuleVersionsResponse,
};
use abstract_sdk::feature_objects::VersionControlContract;
use abstract_sdk::ModuleRegistryInterface;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, Env, Order, QueryRequest, StdError, StdResult, WasmQuery,
};
use cw2::ContractVersion;
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
    let account_id = ACCOUNT_ID.load(deps.storage)?;
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

    let config = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract::new(config.version_control_address);

    let mut resp_vec: Vec<ManagerModuleInfo> = vec![];
    for (id, address) in ids_and_addr.into_iter() {
        let version = query_module_version(deps, address.clone(), &version_control)?;
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
    deps: Deps,
    module_addr: Addr,
    version_control: &VersionControlContract,
) -> StdResult<ContractVersion> {
    let module_registry = version_control.module_registry(deps);

    if let Ok(info) = cw2::query_contract_info(&deps.querier, module_addr.to_string()) {
        // Check if it's abstract format and return now
        if ModuleInfo::from_id(
            &info.contract,
            module::ModuleVersion::Version(info.version.clone()),
        )
        .is_ok()
        {
            return Ok(info);
        }
    }
    // Right now we have either
    // - failed cw2 query
    // - the query succeeded but the cw2 name doesn't adhere to our formatting standards
    //
    // Which means this contract is standalone Trying to get it from VC
    let code_id = deps
        .querier
        .query_wasm_contract_info(module_addr.to_string())?
        .code_id;
    module_registry
        .query_standalone_info(code_id)
        .and_then(|module_info| {
            let module_info =
                module_info.ok_or(abstract_sdk::AbstractSdkError::StandaloneNotFound {
                    code_id,
                    registry_addr: version_control.address.clone(),
                })?;
            let version = ContractVersion::try_from(module_info)?;
            Ok(version)
        })
        .map_err(|e| StdError::generic_err(e.to_string()))
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

    let config = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract::new(config.version_control_address);
    for (name, address) in addresses.into_iter() {
        let result = query_module_version(deps, address, &version_control)?;
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
