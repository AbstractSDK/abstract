use crate::error::VCError;
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::get_contract_version;
use cw2::set_contract_version;
use pandora_os::native::version_control::state::SUBSCRIPTION;
use pandora_os::registery::VERSION_CONTROL;
use pandora_os::util::admin::authorized_set_admin;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::commands::*;
use crate::queries;
use pandora_os::native::version_control::state::{ADMIN, FACTORY};
use pandora_os::native::version_control::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

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
    // Setup the admin as the creator of the contract
    ADMIN.set(deps.branch(), Some(info.sender))?;

    FACTORY.set(deps, None)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> VCResult {
    match msg {
        ExecuteMsg::AddCodeId {
            module,
            version,
            code_id,
        } => add_code_id(deps, info, module, version, code_id),
        ExecuteMsg::RemoveCodeId { module, version } => remove_code_id(deps, info, module, version),
        ExecuteMsg::AddOs {
            os_id,
            manager_address,
            proxy_address,
        } => add_os(deps, info, os_id, manager_address, proxy_address),
        ExecuteMsg::RemoveDebtors { os_ids } => remove_debtors(deps, info, os_ids),
        ExecuteMsg::SetAdmin { new_admin } => set_admin(deps, info, new_admin),
        ExecuteMsg::SetFactory { new_factory } => authorized_set_admin(deps, info, &ADMIN,&FACTORY,new_factory).map_err(|e| e.into()),
        ExecuteMsg::SetSubscription { new_sub_contract } => authorized_set_admin(deps, info, &ADMIN,&SUBSCRIPTION,new_sub_contract).map_err(|e| e.into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // TODO: Add query to get latest version and code_id for some module
        // That way we don't need to hard-code versions in factory contract
        QueryMsg::QueryEnabledModules { os_address } => {
            queries::query_enabled_modules(deps, deps.api.addr_validate(&os_address)?)
        }
        QueryMsg::QueryOsAddress { os_id } => queries::query_os_address(deps, os_id),
        QueryMsg::QueryCodeId { module } => queries::query_code_id(deps, module),
    }
}
