use cosmwasm_std::{DepsMut, MessageInfo, Response};
use cw_controllers::{Admin, AdminError};

pub fn authorized_set_admin<
    C: std::clone::Clone + std::fmt::Debug + std::cmp::PartialEq + schemars::JsonSchema,
>(
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
