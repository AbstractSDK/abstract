use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response, StdResult};

use crate::contract::VCResult;
use crate::error::VersionError;
use crate::state::*;
use dao_os::version_control::msg::ExecuteMsg;

/// Handles the common base execute messages
pub fn handle_message(deps: DepsMut, info: MessageInfo, message: ExecuteMsg) -> VCResult {
    match message {
        ExecuteMsg::AddCodeId { module, version, code_id } => add_code_id(deps, info, module, version, code_id),
        ExecuteMsg::RemoveCodeId { module, version } => remove_code_id(deps, info, module, version),
        
    }
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

/// Add a new code_id for a module
pub fn add_code_id(
    deps: DepsMut,
    msg_info: MessageInfo,
    module: String,
    version: String,
    code_id: u64,
) -> VCResult {
    // Only Admin can update code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    MODULE_CODE_IDS.save(deps.storage, (&module, &version), &code_id)?;

    Ok(Response::new().add_attributes(vec![
        ("Action", "Add Code_ID"),
        ("Module:", &module),
        ("Version:", &version),
        ("Code ID:", &code_id.to_string()),
    ]))
}

/// Add a new code_id for a module
pub fn remove_code_id(
    deps: DepsMut,
    msg_info: MessageInfo,
    module: String,
    version: String,
) -> VCResult {
    // Only Admin can update code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    if MODULE_CODE_IDS.has(deps.storage, (&module, &version)) {
        MODULE_CODE_IDS.remove(deps.storage, (&module, &version));
    } else {
        return Err(VersionError::MissingCodeId {
            module,
            version,
        })
    }

    Ok(Response::new().add_attributes(vec![
        ("Action", "Remove Code_ID"),
        ("Module:", &module),
        ("Version:", &version),
    ]))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> VCResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}
