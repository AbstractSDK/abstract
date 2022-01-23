use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::commands::*;
use crate::error::ManagerError;
use crate::queries;
use crate::state::{ADMIN, OS_ID};
use pandora::manager::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use pandora::registery::MANAGER;

pub type ManagerResult = Result<Response, ManagerError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;

    OS_ID.save(deps.storage, &msg.os_id)?;
    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ManagerResult {
    handle_message(deps, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryVersions { names } => {
            queries::handle_contract_versions_query(deps, env, names)
        }
        QueryMsg::QueryModules { names } => {
            queries::handle_module_addresses_query(deps, env, names)
        }
        QueryMsg::QueryEnabledModules {} => queries::handle_enabled_modules_query(deps),

        QueryMsg::QueryOsId {} => to_binary(&OS_ID.load(deps.storage)?),
    }
}
