use crate::error::VCError;
use abstract_core::version_control::Config;
use abstract_sdk::core::{
    objects::{module_version::migrate_module_data, module_version::set_module_data},
    version_control::{
        state::{CONFIG, FACTORY},
        ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    VERSION_CONTROL,
};
use abstract_sdk::{execute_update_ownership, query_ownership};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::{get_contract_version, set_contract_version};

use cw_semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::commands::*;
use crate::queries;

pub type VCResult<T = Response> = Result<T, VCError>;

pub const ABSTRACT_NAMESPACE: &str = "abstract";

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VCResult {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
        migrate_module_data(
            deps.storage,
            VERSION_CONTROL,
            CONTRACT_VERSION,
            None::<String>,
        )?;
    }
    Ok(Response::default())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> VCResult {
    set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        VERSION_CONTROL,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;

    let InstantiateMsg {
        is_testnet,
        namespaces_limit,
    } = msg;

    CONFIG.save(
        deps.storage,
        &Config {
            is_testnet,
            namespaces_limit,
        },
    )?;

    // Set up the admin as the creator of the contract
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    FACTORY.set(deps, None)?;

    Ok(Response::default())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> VCResult {
    match msg {
        ExecuteMsg::AddModules { modules } => add_modules(deps, info, modules),
        ExecuteMsg::ApproveOrRejectModules { approves, rejects } => {
            approve_or_reject_modules(deps, info, approves, rejects)
        }
        ExecuteMsg::RemoveModule { module } => remove_module(deps, info, module),
        ExecuteMsg::YankModule { module } => yank_module(deps, info, module),
        ExecuteMsg::ClaimNamespaces {
            account_id,
            namespaces,
        } => claim_namespaces(deps, info, account_id, namespaces),
        ExecuteMsg::RemoveNamespaces { namespaces } => remove_namespaces(deps, info, namespaces),
        ExecuteMsg::AddAccount {
            account_id,
            account_base: base,
        } => add_account(deps, info, account_id, base),
        ExecuteMsg::UpdateNamespaceLimit { new_limit } => {
            update_namespaces_limit(deps, info, new_limit)
        }
        ExecuteMsg::SetFactory { new_factory } => set_factory(deps, info, new_factory),
        ExecuteMsg::UpdateOwnership(action) => {
            execute_update_ownership!(VcResponse, deps, env, info, action)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AccountBase { account_id } => {
            queries::handle_account_address_query(deps, account_id)
        }
        QueryMsg::Modules { infos } => queries::handle_modules_query(deps, infos),
        QueryMsg::Namespaces { accounts } => queries::handle_namespaces_query(deps, accounts),
        QueryMsg::Config {} => {
            let cw_ownable::Ownership { owner, .. } = cw_ownable::get_ownership(deps.storage)?;

            let factory = FACTORY.get(deps)?.unwrap();
            to_binary(&ConfigResponse {
                admin: owner.unwrap(),
                factory,
            })
        }
        QueryMsg::ModuleList {
            filter,
            start_after,
            limit,
        } => queries::handle_module_list_query(deps, start_after, limit, filter),
        QueryMsg::NamespaceList {
            filter,
            start_after,
            limit,
        } => queries::handle_namespace_list_query(deps, start_after, limit, filter),
        QueryMsg::Ownership {} => query_ownership!(deps),
    }
}
