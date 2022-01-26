use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::contract::ManagerResult;
use crate::error::ManagerError;
use crate::state::*;
use pandora::manager::msg::ExecuteMsg;

pub fn handle_message(deps: DepsMut, info: MessageInfo, message: ExecuteMsg) -> ManagerResult {
    match message {
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateModuleAddresses { to_add, to_remove } => {
            update_module_addresses(deps, info, to_add, to_remove)
        }
    }
}

/// Adds, updates or removes provided addresses.
/// Should only be called by contract that adds/removes modules.
/// Factory is admin on init
/// TODO: Add functionality to version_control (or some other contract) to add and upgrade contracts.
pub fn update_module_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Option<Vec<(String, String)>>,
    to_remove: Option<Vec<String>>,
) -> ManagerResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    if let Some(modules_to_add) = to_add {
        for (name, new_address) in modules_to_add.into_iter() {
            if name.len() == 0 {
                return Err(ManagerError::InvalidModuleName {});
            };
            // validate addr
            deps.as_ref().api.addr_validate(&new_address)?;
            OS_MODULES.save(deps.storage, name.as_str(), &new_address)?;
        }
    }

    if let Some(modules_to_remove) = to_remove {
        for name in modules_to_remove.into_iter() {
            OS_MODULES.remove(deps.storage, name.as_str());
        }
    }

    Ok(Response::new().add_attribute("action", "update OS module addresses"))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> ManagerResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}
