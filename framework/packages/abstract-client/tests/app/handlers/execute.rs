use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use super::super::contract::{App, AppResult};

use super::super::msg::AppExecuteMsg;
use super::super::state::CONFIG;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: App) -> AppResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.tag_response(Response::default(), "update_config"))
}
