use crate::error::VersionError;
use crate::state::FACTORY;
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::get_contract_version;
use cw2::set_contract_version;
use pandora_os::registery::VERSION_CONTROL;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::commands::*;
use crate::queries;
use crate::state::ADMIN;
use pandora_os::native::version_control::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

pub type VCResult = Result<Response, VersionError>;

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
    handle_message(deps, info, msg)
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
