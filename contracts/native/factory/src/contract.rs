use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};
use cw_asset::Asset;
use semver::Version;

use crate::error::OsFactoryError;
use abstract_os::OS_FACTORY;
use cw2::{get_contract_version, set_contract_version};

use crate::{commands, state::*};
use abstract_os::os_factory::*;

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
        version_control_contract: deps.api.addr_validate(&msg.version_control_address)?,
        module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
        memory_contract: deps.api.addr_validate(&msg.memory_address)?,
        subscription_address: None,
        next_os_id: 0u32,
    };

    set_contract_version(deps.storage, OS_FACTORY, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> OsFactoryResult {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateConfig {
            admin,
            memory_contract,
            version_control_contract,
            subscription_address,
            module_factory_address,
        } => commands::execute_update_config(
            deps,
            env,
            info,
            admin,
            memory_contract,
            version_control_contract,
            module_factory_address,
            subscription_address,
        ),
        ExecuteMsg::CreateOs {
            governance,
            link,
            name,
            description,
        } => {
            let maybe_received_coin = info.funds.last().map(Asset::from);
            commands::execute_create_os(
                deps,
                env,
                governance,
                maybe_received_coin,
                name,
                description,
                link,
            )
        }
    }
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> OsFactoryResult {
    match msg {
        Reply {
            id: commands::CREATE_OS_MANAGER_MSG_ID,
            result,
        } => commands::after_manager_create_proxy(deps, result),
        Reply {
            id: commands::CREATE_OS_TREASURY_MSG_ID,
            result,
        } => commands::after_proxy_add_to_manager_and_set_admin(deps, result),
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
        subscription_address: state.subscription_address.map(Addr::into),
        module_factory_address: state.module_factory_address.into(),
        next_os_id: state.next_os_id,
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
