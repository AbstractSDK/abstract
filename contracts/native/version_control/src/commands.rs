use cosmwasm_std::{DepsMut, Empty, MessageInfo, Response};

use crate::contract::VCResult;
use crate::error::VCError;
use abstract_os::native::version_control::state::*;

/// Add new OS to version control contract
/// Only Factory can add OS
pub fn add_os(
    deps: DepsMut,
    msg_info: MessageInfo,
    os_id: u32,
    os_manager: String,
    os_proxy: String,
) -> VCResult {
    // Only Factory can add new OS
    FACTORY.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let manager = deps.api.addr_validate(&os_manager)?;
    let proxy = deps.api.addr_validate(&os_proxy)?;
    OS_ADDRESSES.save(deps.storage, os_id, &Core { manager, proxy })?;

    Ok(Response::new().add_attributes(vec![
        ("Action", "Add OS"),
        ("ID:", &os_id.to_string()),
        ("Manager:", &os_manager),
        ("Proxy", &os_proxy),
    ]))
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
        return Err(VCError::MissingCodeId { module, version });
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
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

// Might be useful later to manage state bloat.

// /// Remove OS from version control contract
// pub fn remove_debtors(deps: DepsMut, msg_info: MessageInfo, os_ids: Vec<u32>) -> VCResult {
//     // Only Admin can update code-ids
//     SUBSCRIPTION.assert_admin(deps.as_ref(), &msg_info.sender)?;

//     for os_id in os_ids {
//         if OS_ADDRESSES.has(deps.storage, os_id) {
//             OS_ADDRESSES.remove(deps.storage, os_id);
//         } else {
//             return Err(VCError::MissingOsId { id: os_id });
//         }
//     }

//     Ok(Response::new())
// }
