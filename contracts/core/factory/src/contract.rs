use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

use crate::error::OsFactoryError;
use cw2::set_contract_version;
use pandora::registery::FACTORY;

use crate::state::*;
use crate::{commands, msg::*};

pub type OsFactoryResult = Result<Response, OsFactoryError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> OsFactoryResult {
    let config = Config {
        version_control_contract: deps.api.addr_validate(&msg.version_control_contract)?,
        memory_contract: deps.api.addr_validate(&msg.memory_contract)?,
        creation_fee: msg.creation_fee,
        next_os_id: 0u32,
    };

    set_contract_version(deps.storage, FACTORY, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> OsFactoryResult {
    match msg {
        ExecuteMsg::UpdateConfig {
            admin,
            memory_contract,
            version_control_contract,
            creation_fee,
        } => commands::execute_update_config(
            deps,
            env,
            info,
            admin,
            memory_contract,
            version_control_contract,
            creation_fee,
        ),
        ExecuteMsg::CreateOs { governance } => commands::execute_create_os(deps, env, governance),
    }
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> OsFactoryResult {
    match msg {
        Reply {
            id: commands::CREATE_OS_MANAGER_MSG_ID,
            result,
        } => commands::after_manager_create_treasury(deps, result),
        Reply {
            id: commands::CREATE_OS_TREASURY_MSG_ID,
            result,
        } => commands::after_treasury_add_to_manager(deps, result),
        _ => Err(OsFactoryError::UnexpectedReply {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    let resp = ConfigResponse {
        owner: admin.into(),
        version_control_contract: state.version_control_contract.into(),
        memory_contract: state.memory_contract.into(),
        creation_fee: state.creation_fee,
        next_os_id: state.next_os_id,
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
