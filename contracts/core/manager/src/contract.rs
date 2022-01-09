use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::commands::*;
use crate::error::ManagerError;
use crate::queries;
use crate::state::{ADMIN, OS_ID};
use dao_os::manager::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub type ManagerResult = Result<Response, ManagerError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
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
        QueryMsg::QueryEnabledModules {} => 
            queries::handle_enabled_modules_query(deps)
    }
}
