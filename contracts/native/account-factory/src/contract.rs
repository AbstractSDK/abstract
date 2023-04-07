use crate::{commands, error::AccountFactoryError, state::*};
use abstract_sdk::core::{
    account_factory::*,
    objects::module_version::{migrate_module_data, set_module_data},
    ACCOUNT_FACTORY,
};
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use cw_asset::Asset;
use semver::Version;

pub type AccountFactoryResult = Result<Response, AccountFactoryError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> AccountFactoryResult {
    let config = Config {
        version_control_contract: deps.api.addr_validate(&msg.version_control_address)?,
        module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
        ans_host_contract: deps.api.addr_validate(&msg.ans_host_address)?,
        next_account_id: 0u32,
    };

    set_contract_version(deps.storage, ACCOUNT_FACTORY, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        ACCOUNT_FACTORY,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;

    CONFIG.save(deps.storage, &config)?;
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> AccountFactoryResult {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateConfig {
            admin,
            ans_host_contract,
            version_control_contract,
            module_factory_address,
        } => commands::execute_update_config(
            deps,
            env,
            info,
            admin,
            ans_host_contract,
            version_control_contract,
            module_factory_address,
        ),
        ExecuteMsg::CreateAccount {
            governance,
            link,
            name,
            description,
        } => {
            let maybe_received_coin = info.funds.last().map(Asset::from);
            let gov_details = governance.verify(deps.api)?;
            commands::execute_create_account(
                deps,
                env,
                gov_details,
                maybe_received_coin,
                name,
                description,
                link,
            )
        }
    }
}

/// This just stores the result for future query
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> AccountFactoryResult {
    match msg {
        Reply {
            id: commands::CREATE_ACCOUNT_MANAGER_MSG_ID,
            result,
        } => commands::after_manager_create_proxy(deps, result),
        Reply {
            id: commands::CREATE_ACCOUNT_PROXY_MSG_ID,
            result,
        } => commands::after_proxy_add_to_manager_and_set_admin(deps, result),
        _ => Err(AccountFactoryError::UnexpectedReply {}),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
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
        ans_host_contract: state.ans_host_contract.into(),
        module_factory_address: state.module_factory_address.into(),
        next_account_id: state.next_account_id,
    };

    Ok(resp)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();

    if storage_version < version {
        set_contract_version(deps.storage, ACCOUNT_FACTORY, CONTRACT_VERSION)?;
        migrate_module_data(
            deps.storage,
            ACCOUNT_FACTORY,
            CONTRACT_VERSION,
            None::<String>,
        )?;
    }
    Ok(Response::default())
}
