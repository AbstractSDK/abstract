use crate::error::VCError;
use abstract_sdk::os::VERSION_CONTROL;
use cosmwasm_std::to_binary;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::get_contract_version;
use cw2::set_contract_version;
use cw_controllers::{Admin, AdminError};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::commands::*;
use crate::queries;
use abstract_sdk::os::version_control::state::{ADMIN, FACTORY};
use abstract_sdk::os::version_control::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

pub type VCResult = Result<Response, VCError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VCResult {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> VCResult {
    set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    // Setup the admin as the creator of the contract
    ADMIN.set(deps.branch(), Some(info.sender))?;
    FACTORY.set(deps, None)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> VCResult {
    match msg {
        ExecuteMsg::AddModules { modules } => add_modules(deps, info, modules),
        ExecuteMsg::RemoveModule { module } => remove_module(deps, info, module),
        ExecuteMsg::AddOs { os_id, core } => add_os(deps, info, os_id, core),
        ExecuteMsg::SetAdmin { new_admin } => set_admin(deps, info, new_admin),
        ExecuteMsg::SetFactory { new_factory } => {
            authorized_set_admin(deps, info, &ADMIN, &FACTORY, new_factory).map_err(|e| e.into())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OsCore { os_id } => queries::handle_os_address_query(deps, os_id),
        QueryMsg::Module { module } => queries::handle_module_query(deps, module),
        QueryMsg::Config {} => {
            let admin = ADMIN.get(deps)?.unwrap().into_string();
            let factory = FACTORY.get(deps)?.unwrap().into_string();
            to_binary(&ConfigResponse { admin, factory })
        }
        QueryMsg::Modules {
            page_token,
            page_size,
        } => queries::handle_modules_query(deps, page_token, page_size),
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
