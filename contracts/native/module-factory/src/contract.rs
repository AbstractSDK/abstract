use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use semver::Version;

use crate::error::ModuleFactoryError;
use abstract_sdk::os::OS_FACTORY;
use cw2::{get_contract_version, set_contract_version};

use crate::{commands, state::*};
use abstract_sdk::os::module_factory::*;

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
        ans_host_address: deps.api.addr_validate(&msg.ans_host_address)?,
    };

    set_contract_version(deps.storage, OS_FACTORY, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            core: None,
            module: None,
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
            ans_host_address,
            version_control_address,
        } => commands::execute_update_config(
            deps,
            env,
            info,
            admin,
            ans_host_address,
            version_control_address,
        ),
        ExecuteMsg::InstallModule { module, init_msg } => {
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
            id: commands::CREATE_APP_RESPONSE_ID,
            result,
        } => commands::register_contract(deps, result),
        Reply {
            id: commands::CREATE_SERVICE_RESPONSE_ID,
            result,
        } => commands::register_contract(deps, result),
        Reply {
            id: commands::CREATE_PERK_RESPONSE_ID,
            result,
        } => commands::register_contract(deps, result),
        _ => Err(ModuleFactoryError::UnexpectedReply {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Context {} => to_binary(&query_context(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    let resp = ConfigResponse {
        owner: admin.into(),
        version_control_address: state.version_control_address.into(),
        ans_host_address: state.ans_host_address.into(),
    };

    Ok(resp)
}

pub fn query_context(deps: Deps) -> StdResult<ContextResponse> {
    let context: Context = CONTEXT.load(deps.storage)?;
    let resp = ContextResponse {
        core: context.core,
        module: context.module,
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, OS_FACTORY, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}
