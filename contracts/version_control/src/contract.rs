use crate::error::VersionError;
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::commands::*;
use crate::queries;
use crate::state::ADMIN;
use dao_os::version_control::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub type VCResult = Result<Response, VersionError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, _msg: InstantiateMsg) -> VCResult {
    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> VCResult {
    handle_message(deps, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryEnabledModules { os_address } => {
            queries::query_enabled_modules(deps, deps.api.addr_validate(&os_address)?)
        }
        QueryMsg::QueryOsAddress { os_id } => {
            queries::query_os_address(deps, os_id)
        }
    }
}
