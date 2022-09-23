use cosmwasm_std::{DepsMut, Empty, MessageInfo, Response};

use crate::contract::VCResult;
use crate::error::VCError;
use abstract_os::{
    objects::module::ModuleInfo,
    version_control::{state::*, Core},
};

/// Add new OS to version control contract
/// Only Factory can add OS
pub fn add_os(deps: DepsMut, msg_info: MessageInfo, os_id: u32, core: Core) -> VCResult {
    // Only Factory can add new OS
    FACTORY.assert_admin(deps.as_ref(), &msg_info.sender)?;
    OS_ADDRESSES.save(deps.storage, os_id, &core)?;

    Ok(Response::new().add_attributes(vec![
        ("Action", "Add OS"),
        ("ID:", &os_id.to_string()),
        ("Manager:", core.manager.as_ref()),
        ("Proxy", core.proxy.as_ref()),
    ]))
}

/// Add a new code_id for a module
/// Here we can add logic to allow subscribers to claim a namespace and upload contracts to that namespace
pub fn add_code_ids(
    deps: DepsMut,
    msg_info: MessageInfo,
    code_ids: Vec<(ModuleInfo, u64)>,
) -> VCResult {
    // Only Admin can update code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    for (module, code_id) in code_ids {
        if MODULE_CODE_IDS.has(deps.storage, module.clone()) {
            return Err(VCError::CodeIdUpdate(module));
        }
        // version must be set in order to add the new version
        module.assert_version_variant()?;

        MODULE_CODE_IDS.save(deps.storage, module, &code_id)?;
    }

    Ok(Response::new().add_attributes(vec![("Action", "Add Code_IDs")]))
}

/// Remove a code_id for a module
pub fn remove_code_id(deps: DepsMut, msg_info: MessageInfo, module: ModuleInfo) -> VCResult {
    // Only Admin can update code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    module.assert_version_variant()?;
    if MODULE_CODE_IDS.has(deps.storage, module.clone()) {
        MODULE_CODE_IDS.remove(deps.storage, module.clone());
    } else {
        return Err(VCError::MissingCodeId(module));
    }

    Ok(Response::new().add_attributes(vec![
        ("action", "remove code_id"),
        ("module:", &module.to_string()),
    ]))
}

/// Add a new code_id for a module
pub fn add_apis(
    deps: DepsMut,
    msg_info: MessageInfo,
    addresses: Vec<(ModuleInfo, String)>,
) -> VCResult {
    // Only Admin can add code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    for (module, address) in addresses {
        if API_ADDRESSES.has(deps.storage, module.clone()) {
            return Err(VCError::ApiUpdate(module));
        }
        module.assert_version_variant()?;
        API_ADDRESSES.save(deps.storage, module, &deps.api.addr_validate(&address)?)?;
    }

    Ok(Response::new().add_attributes(vec![("Action", "Add Code_ID")]))
}

/// Add a new code_id for a module
pub fn remove_api(deps: DepsMut, msg_info: MessageInfo, module: ModuleInfo) -> VCResult {
    // Only Admin can update code-ids
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    module.assert_version_variant()?;

    if API_ADDRESSES.has(deps.storage, module.clone()) {
        API_ADDRESSES.remove(deps.storage, module.clone());
    } else {
        return Err(VCError::MissingApi(module));
    }

    Ok(Response::new().add_attributes(vec![
        ("action", "remove code_id"),
        ("module:", &module.to_string()),
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
