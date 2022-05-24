use abstract_os::core::modules::ModuleInfo;
use cosmwasm_std::Order;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::StdError;
use cosmwasm_std::Uint64;
use cosmwasm_std::WasmQuery;
use cw2::ContractVersion;

use crate::error::VCError;
use abstract_os::native::version_control::state::{MODULE_CODE_IDS, OS_ADDRESSES};
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};
use cw_storage_plus::U32Key;

use abstract_os::core::manager::msg::{EnabledModulesResponse, QueryMsg};
use abstract_os::native::version_control::msg::CodeIdResponse;

pub fn query_enabled_modules(deps: Deps, manager_addr: Addr) -> StdResult<Binary> {
    let response: EnabledModulesResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: manager_addr.to_string(),
            msg: to_binary(&QueryMsg::QueryEnabledModules {})?,
        }))?;
    to_binary(&response)
}

pub fn query_os_address(deps: Deps, os_id: u32) -> StdResult<Binary> {
    let os_address = OS_ADDRESSES.load(deps.storage, U32Key::new(os_id));
    match os_address {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingOsId { id: os_id }.to_string(),
        )),
        Ok(address) => to_binary(&address),
    }
}

pub fn query_code_id(deps: Deps, module: ModuleInfo) -> StdResult<Binary> {
    // Will store actual version of returned module code id
    let resulting_version: String;

    let code_id = if let Some(version) = module.version.clone() {
        resulting_version = version.clone();
        MODULE_CODE_IDS.load(deps.storage, (&module.name, &version))
    } else {
        // get latest
        let versions: StdResult<Vec<(Vec<u8>, u64)>> = MODULE_CODE_IDS
            .prefix(&module.name)
            .range(deps.storage, None, None, Order::Descending)
            .take(1)
            .collect();
        let (latest_version, id) = &versions?[0];
        resulting_version = std::str::from_utf8(latest_version)?.to_owned();
        Ok(*id)
    };

    match code_id {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingCodeId {
                module: module.name,
                version: module.version.unwrap_or_else(|| "".to_string()),
            }
            .to_string(),
        )),
        Ok(id) => to_binary(&CodeIdResponse {
            code_id: Uint64::from(id),
            info: ContractVersion {
                version: resulting_version,
                contract: module.name,
            },
        }),
    }
}
