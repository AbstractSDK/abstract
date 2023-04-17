use crate::{commands, error::ModuleFactoryError, state::*};
use abstract_core::objects::module_version::migrate_module_data;
use abstract_macros::abstract_response;
use abstract_sdk::core::{
    module_factory::*, objects::module_version::set_module_data, MODULE_FACTORY,
};
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

pub type ModuleFactoryResult<T = Response> = Result<T, ModuleFactoryError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Abstract module factory response
#[abstract_response(MODULE_FACTORY)]
pub struct ModuleFactoryResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
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

    set_contract_version(deps.storage, MODULE_FACTORY, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        MODULE_FACTORY,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;
    CONFIG.save(deps.storage, &config)?;
    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            account_base: None,
            module: None,
        },
    )?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(Response::new())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ModuleFactoryResult {
    match msg {
        ExecuteMsg::UpdateConfig {
            ans_host_address,
            version_control_address,
        } => commands::execute_update_config(
            deps,
            env,
            info,
            ans_host_address,
            version_control_address,
        ),
        ExecuteMsg::InstallModule { module, init_msg } => {
            commands::execute_create_module(deps, env, info, module, init_msg)
        }
        ExecuteMsg::UpdateFactoryBinaryMsgs { to_add, to_remove } => {
            commands::update_factory_binaries(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateOwnership(action) => {
            abstract_sdk::execute_update_ownership!(ModuleFactoryResponse, deps, env, info, action)
        }
    }
}

/// This just stores the result for future query
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ModuleFactoryResult {
    match msg {
        Reply {
            id: commands::CREATE_APP_RESPONSE_ID,
            result,
        } => commands::register_contract(deps, result),
        Reply {
            id: commands::CREATE_STANDALONE_RESPONSE_ID,
            result,
        } => commands::register_contract(deps, result),
        _ => Err(ModuleFactoryError::UnexpectedReply {}),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Context {} => to_binary(&query_context(deps)?),
        QueryMsg::Ownership {} => abstract_sdk::query_ownership!(deps),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let cw_ownable::Ownership { owner, .. } = cw_ownable::get_ownership(deps.storage)?;
    let resp = ConfigResponse {
        owner: owner.unwrap(),
        version_control_address: state.version_control_address.into(),
        ans_host_address: state.ans_host_address.into(),
    };

    Ok(resp)
}

pub fn query_context(deps: Deps) -> StdResult<ContextResponse> {
    let Context {
        account_base,
        module,
    }: Context = CONTEXT.load(deps.storage)?;
    let resp = ContextResponse {
        account_base,
        module,
    };

    Ok(resp)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, MODULE_FACTORY, CONTRACT_VERSION)?;
        migrate_module_data(
            deps.storage,
            MODULE_FACTORY,
            CONTRACT_VERSION,
            None::<String>,
        )?;
    }
    Ok(Response::default())
}
