use cosmwasm_std::QueryRequest;
use cosmwasm_std::StdError;
use cosmwasm_std::Uint64;
use cosmwasm_std::WasmQuery;

use crate::error::VersionError;
use crate::state::{MODULE_CODE_IDS, OS_ADDRESSES};
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};
use cw_storage_plus::U32Key;

use pandora::manager::msg::{EnabledModulesResponse, QueryMsg};
use pandora::version_control::msg::CodeIdResponse;

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
        Err(_) => {
            return Err(StdError::generic_err(
                VersionError::MissingOsId { id: os_id }.to_string(),
            ))
        }
        Ok(address) => to_binary(&address),
    }
}

pub fn query_code_id(deps: Deps, module: String, version: String) -> StdResult<Binary> {
    let code_id = MODULE_CODE_IDS.load(deps.storage, (&module, &version));

    match code_id {
        Err(_) => {
            return Err(StdError::generic_err(
                VersionError::MissingCodeId { module, version }.to_string(),
            ))
        }
        Ok(id) => to_binary(&CodeIdResponse {
            code_id: Uint64::from(id),
        }),
    }
}
