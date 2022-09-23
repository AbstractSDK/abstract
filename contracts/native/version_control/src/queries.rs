use abstract_os::objects::module::ModuleInfo;
use abstract_os::objects::module::ModuleVersion;
use abstract_os::version_control::state::API_ADDRESSES;
use abstract_os::version_control::ApiAddressResponse;
use abstract_os::version_control::ApiAddressesResponse;
use abstract_os::version_control::CodeIdsResponse;
use abstract_os::version_control::OsCoreResponse;
use cosmwasm_std::Order;
use cosmwasm_std::StdError;
use cosmwasm_std::Uint64;
use cw_storage_plus::Bound;

use crate::error::VCError;
use abstract_os::version_control::state::{MODULE_CODE_IDS, OS_ADDRESSES};
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use abstract_os::version_control::CodeIdResponse;

const DEFAULT_LIMIT: u8 = 10;
const MAX_LIMIT: u8 = 20;

pub fn handle_os_address_query(deps: Deps, os_id: u32) -> StdResult<Binary> {
    let os_address = OS_ADDRESSES.load(deps.storage, os_id);
    match os_address {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingOsId { id: os_id }.to_string(),
        )),
        Ok(core) => to_binary(&OsCoreResponse { os_core: core }),
    }
}

pub fn handle_code_id_query(deps: Deps, mut module: ModuleInfo) -> StdResult<Binary> {
    let maybe_code_id = if let ModuleVersion::Version(_) = module.version {
        MODULE_CODE_IDS.load(deps.storage, module.clone())
    } else {
        // get latest
        let versions: StdResult<Vec<(String, u64)>> = MODULE_CODE_IDS
            .prefix((module.provider.clone(), module.name.clone()))
            .range(deps.storage, None, None, Order::Descending)
            .take(1)
            .collect();
        let (latest_version, id) = versions?
            .first()
            .ok_or(StdError::GenericErr {
                msg: format!("code id for {} not found", module),
            })?
            .clone();
        module.version = ModuleVersion::Version(latest_version);
        Ok(id)
    };

    match maybe_code_id {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingCodeId(module).to_string(),
        )),
        Ok(id) => to_binary(&CodeIdResponse {
            code_id: Uint64::from(id),
            info: module,
        }),
    }
}

pub fn handle_api_address_query(deps: Deps, mut module: ModuleInfo) -> StdResult<Binary> {
    let maybe_addr = if let ModuleVersion::Version(_) = module.version {
        API_ADDRESSES.load(deps.storage, module.clone())
    } else {
        // get latest
        let versions: StdResult<Vec<(String, Addr)>> = API_ADDRESSES
            .prefix((module.provider.clone(), module.name.clone()))
            .range(deps.storage, None, None, Order::Descending)
            .take(1)
            .collect();
        let (latest_version, addr) = versions?
            .first()
            .ok_or(StdError::GenericErr {
                msg: format!("api module {} not available", module),
            })?
            .clone();
        module.version = ModuleVersion::Version(latest_version);
        Ok(addr)
    };

    match maybe_addr {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingCodeId(module).to_string(),
        )),
        Ok(address) => to_binary(&ApiAddressResponse {
            address,
            info: module,
        }),
    }
}

pub fn handle_code_ids_query(
    deps: Deps,
    page_token: Option<ModuleInfo>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound: Option<Bound<ModuleInfo>> = page_token.map(Bound::exclusive);

    let res: Result<Vec<(ModuleInfo, u64)>, _> = MODULE_CODE_IDS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&CodeIdsResponse {
        module_code_ids: res?,
    })
}

pub fn handle_api_addresses_query(
    deps: Deps,
    page_token: Option<ModuleInfo>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound: Option<Bound<ModuleInfo>> = page_token.map(Bound::exclusive);
    let res: Result<Vec<(ModuleInfo, Addr)>, _> = API_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&ApiAddressesResponse {
        api_addresses: res?
            .into_iter()
            .map(|(module, addr)| (module, addr.into_string()))
            .collect(),
    })
}
