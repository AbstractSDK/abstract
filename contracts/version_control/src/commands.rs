use cosmwasm_std::{DepsMut, MessageInfo, Response};
use cw_storage_plus::U32Key;

use crate::contract::VCResult;
use crate::error::VersionError;
use crate::state::*;
use pandora::version_control::msg::ExecuteMsg;

/// Handles the common base execute messages
pub fn handle_message(deps: DepsMut, info: MessageInfo, message: ExecuteMsg) -> VCResult {
    match message {
        ExecuteMsg::AddCodeId {
            module,
            version,
            code_id,
        } => add_code_id(deps, info, module, version, code_id),
        ExecuteMsg::RemoveCodeId { module, version } => remove_code_id(deps, info, module, version),
        ExecuteMsg::AddOs {
            os_id,
            os_manager_address,
        } => add_os(deps, info, os_id, os_manager_address),
        ExecuteMsg::RemoveOs { os_id } => remove_os(deps, info, os_id),
        ExecuteMsg::SetAdmin { new_admin } => set_admin(deps, info, new_admin),
        ExecuteMsg::SetFactory { new_factory } => set_factory(deps, info, new_factory),
    }
}

/// Add new OS to version control contract
/// Only Factory can add OS
pub fn add_os(deps: DepsMut, msg_info: MessageInfo, os_id: u32, os_manager: String) -> VCResult {
    // Only Factory can add new OS
    FACTORY.assert_admin(deps.as_ref(), &msg_info.sender)?;

    deps.api.addr_validate(&os_manager)?;
    OS_ADDRESSES.save(deps.storage, U32Key::from(os_id), &os_manager)?;

    Ok(Response::new().add_attributes(vec![
        ("Action", "Add OS"),
        ("ID:", &os_id.to_string()),
        ("OS Address:", &os_manager),
    ]))
}

/// Remove OS from version control contract
pub fn remove_os(deps: DepsMut, msg_info: MessageInfo, os_id: u32) -> VCResult {
    // Only Admin can update code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    if OS_ADDRESSES.has(deps.storage, U32Key::from(os_id)) {
        OS_ADDRESSES.remove(deps.storage, U32Key::from(os_id));
    } else {
        return Err(VersionError::MissingOsId { id: os_id });
    }

    Ok(Response::new().add_attributes(vec![("Action", "Remove OS"), ("ID:", &os_id.to_string())]))
}

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
        return Err(VersionError::MissingCodeId { module, version });
    }

    Ok(Response::new().add_attributes(vec![
        ("Action", "Remove Code_ID"),
        ("Module:", &module),
        ("Version:", &version),
    ]))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> VCResult {
    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    // Admin is asserted here
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

pub fn set_factory(deps: DepsMut, info: MessageInfo, factory: String) -> VCResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let factory_addr = deps.api.addr_validate(&factory)?;
    FACTORY.set(deps, Some(factory_addr))?;
    Ok(Response::default().add_attribute("new factory", factory))
}
