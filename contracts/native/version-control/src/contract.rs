use crate::error::VCError;
use abstract_sdk::core::{
    objects::{module_version::migrate_module_data, module_version::set_module_data},
    version_control::{
        state::{ADMIN, FACTORY},
        ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    VERSION_CONTROL,
};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::{get_contract_version, set_contract_version};
use cw_controllers::{Admin, AdminError};
use cw_semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
use crate::commands::*;
use crate::queries;

pub type VCResult = Result<Response, VCError>;

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
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> VCResult {
    set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        VERSION_CONTROL,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;
    // Setup the admin as the creator of the contract
    ADMIN.set(deps.branch(), Some(info.sender))?;
    FACTORY.set(deps, None)?;

    Ok(Response::default())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> VCResult {
    match msg {
        ExecuteMsg::AddModules { modules } => add_modules(deps, info, modules),
        ExecuteMsg::RemoveModule { module } => remove_module(deps, info, module),
        ExecuteMsg::AddAccount {
            account_id,
            account_base: base,
        } => add_account(deps, info, account_id, base),
        ExecuteMsg::SetAdmin { new_admin } => set_admin(deps, info, new_admin),
        ExecuteMsg::SetFactory { new_factory } => {
            authorized_set_admin(deps, info, &ADMIN, &FACTORY, new_factory).map_err(|e| e.into())
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
        QueryMsg::Config {} => {
            let admin = ADMIN.get(deps)?.unwrap().into_string();
            let factory = FACTORY.get(deps)?.unwrap().into_string();
            to_binary(&ConfigResponse { admin, factory })
        }
        QueryMsg::ModuleList {
            filter,
            start_after,
            limit,
        } => queries::handle_module_list_query(deps, start_after, limit, filter),
    }
}

fn authorized_set_admin<C: std::clone::Clone + std::fmt::Debug + std::cmp::PartialEq>(
    deps: DepsMut,
    info: MessageInfo,
    authorized_user: &Admin,
    admin_to_update: &Admin,
    new_admin: String,
) -> Result<Response<C>, AdminError> {
    authorized_user.assert_admin(deps.as_ref(), &info.sender)?;

    let new_admin_addr = deps.api.addr_validate(&new_admin)?;
    admin_to_update.set(deps, Some(new_admin_addr))?;
    Ok(Response::new().add_attribute("Set admin item to:", new_admin))
}
