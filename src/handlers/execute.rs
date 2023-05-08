use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{TemplateApp, TemplateResult};

use crate::msg::TemplateExecuteMsg;
use crate::state::CONFIG;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: TemplateApp,
    msg: TemplateExecuteMsg,
) -> TemplateResult {
    match msg {
        TemplateExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: TemplateApp) -> TemplateResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.tag_response(Response::default(), "update_config"))
}
