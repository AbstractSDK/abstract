use abstract_os::objects::module::ModuleInfo;
use abstract_os::version_control::state::API_ADDRESSES;
use abstract_os::version_control::QueryApiAddressResponse;
use abstract_os::version_control::QueryApiAddressesResponse;
use abstract_os::version_control::QueryCodeIdsResponse;
use abstract_os::version_control::QueryOsCoreResponse;
use cosmwasm_std::Order;
use cosmwasm_std::StdError;
use cosmwasm_std::Uint64;
use cw2::ContractVersion;
use cw_storage_plus::Bound;

use crate::error::VCError;
use abstract_os::version_control::state::{MODULE_CODE_IDS, OS_ADDRESSES};
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use abstract_os::version_control::QueryCodeIdResponse;

const DEFAULT_LIMIT: u8 = 10;
const MAX_LIMIT: u8 = 20;

pub fn handle_os_address_query(deps: Deps, os_id: u32) -> StdResult<Binary> {
    let os_address = OS_ADDRESSES.load(deps.storage, os_id);
    match os_address {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingOsId { id: os_id }.to_string(),
        )),
        Ok(core) => to_binary(&QueryOsCoreResponse { os_core: core }),
    }
}

pub fn handle_code_id_query(deps: Deps, module: ModuleInfo) -> StdResult<Binary> {
    // Will store actual version of returned module code id
    let resulting_version: String;

    let code_id = if let Some(version) = module.version.clone() {
        resulting_version = version.clone();
        MODULE_CODE_IDS.load(deps.storage, (&module.name, &version))
    } else {
        // get latest
        let versions: StdResult<Vec<(String, u64)>> = MODULE_CODE_IDS
            .prefix(&module.name)
            .range(deps.storage, None, None, Order::Descending)
            .take(1)
            .collect();
        let (latest_version, id) = versions?
            .first()
            .ok_or(StdError::GenericErr {
                msg: format!("code id for {} not found", module),
            })?
            .clone();
        resulting_version = latest_version;
        Ok(id)
    };

    match code_id {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingCodeId {
                module: module.name,
                version: module.version.unwrap_or_default(),
            }
            .to_string(),
        )),
        Ok(id) => to_binary(&QueryCodeIdResponse {
            code_id: Uint64::from(id),
            info: ContractVersion {
                version: resulting_version,
                contract: module.name,
            },
        }),
    }
}

pub fn handle_api_address_query(deps: Deps, module: ModuleInfo) -> StdResult<Binary> {
    // Will store actual version of returned module code id
    let resulting_version: String;

    let maybe_addr = if let Some(version) = module.version.clone() {
        resulting_version = version;
        API_ADDRESSES.load(deps.storage, (&module.name, &resulting_version))
    } else {
        // get latest
        let versions: StdResult<Vec<(String, Addr)>> = API_ADDRESSES
            .prefix(&module.name)
            .range(deps.storage, None, None, Order::Descending)
            .take(1)
            .collect();
        let (latest_version, addr) = versions?
            .first()
            .ok_or(StdError::GenericErr {
                msg: format!("api module {} not available", module),
            })?
            .clone();
        resulting_version = latest_version;
        Ok(addr)
    };

    match maybe_addr {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingCodeId {
                module: module.name,
                version: module.version.unwrap_or_default(),
            }
            .to_string(),
        )),
        Ok(address) => to_binary(&QueryApiAddressResponse {
            address,
            info: ContractVersion {
                version: resulting_version,
                contract: module.name,
            },
        }),
    }
}

pub fn handle_code_ids_query(
    deps: Deps,
    last_module: Option<ContractVersion>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound: Option<Bound<(&str, &str)>> = if last_module.is_some() {
        let ContractVersion { contract, version } = last_module.as_ref().unwrap();
        Some(Bound::exclusive((contract.as_str(), version.as_str())))
    } else {
        None
    };

    let res: Result<Vec<((String, String), u64)>, _> = MODULE_CODE_IDS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    let module_code_ids = res?;
    let mut resp_vec: Vec<(ContractVersion, u64)> = vec![];
    for ((contract, version), code_id) in module_code_ids.into_iter() {
        resp_vec.push((ContractVersion { contract, version }, code_id))
    }

    to_binary(&QueryCodeIdsResponse {
        module_code_ids: resp_vec,
    })
}

pub fn handle_api_addresses_query(
    deps: Deps,
    last_api: Option<ContractVersion>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound: Option<Bound<(&str, &str)>> = if last_api.is_some() {
        let ContractVersion { contract, version } = last_api.as_ref().unwrap();
        Some(Bound::exclusive((contract.as_str(), version.as_str())))
    } else {
        None
    };
    let res: Result<Vec<((String, String), Addr)>, _> = API_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    let module_apis = res?;
    let mut resp_vec: Vec<(ContractVersion, String)> = vec![];
    for ((contract, version), addr) in module_apis.into_iter() {
        resp_vec.push((ContractVersion { contract, version }, addr.to_string()))
    }

    to_binary(&QueryApiAddressesResponse {
        api_addresses: resp_vec,
    })
}
