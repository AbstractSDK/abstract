use cosmwasm_std::{entry_point, Addr};
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

use crate::error::ModuleFactoryError;
use cw2::set_contract_version;
use pandora_os::registery::FACTORY;

use crate::commands;
use crate::state::*;
use pandora_os::native::module_factory::msg::*;

pub type ModuleFactoryResult = Result<Response, ModuleFactoryError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ModuleFactoryResult {
    let config = Config {
        version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
        memory_address: deps.api.addr_validate(&msg.memory_address)?,
    };

    set_contract_version(deps.storage, FACTORY, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            manager: Addr::unchecked(""),
        },
    )?;
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ModuleFactoryResult {
    match msg {
        ExecuteMsg::UpdateConfig {
            admin,
            memory_address,
            version_control_address,
        } => commands::execute_update_config(
            deps,
            env,
            info,
            admin,
            memory_address,
            version_control_address,
        ),
        ExecuteMsg::CreateModule { module, init_msg } => {
            commands::execute_create_module(deps, env, info, module, init_msg)
        }
        ExecuteMsg::UpdateFactoryBinaryMsgs { to_add, to_remove } => {
            commands::update_factory_binaries(deps, info, to_add, to_remove)
        }
    }
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ModuleFactoryResult {
    match msg {
        Reply {
            id: commands::CREATE_INTERNAL_DAPP_RESPONSE_ID,
            result,
        } => commands::handle_internal_dapp_init_result(deps, result),
        Reply {
            id: commands::CREATE_EXTERNAL_DAPP_RESPONSE_ID,
            result,
        } => commands::handle_external_dapp_init_result(deps, result),
        _ => Err(ModuleFactoryError::UnexpectedReply {}),
        // TODO: add admin setters for services and perks
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
        version_control_address: state.version_control_address.into(),
        memory_address: state.memory_address.into(),
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
