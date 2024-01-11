use abstract_app::traits::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo};

use crate::contract::{App, AppResult};

use crate::msg::AppExecuteMsg;
use crate::state::{CONFIG, COUNT};

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::Increment {} => increment(deps, app),
        AppExecuteMsg::Reset { count } => reset(deps, info, count, app),
        AppExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

fn increment(deps: DepsMut, app: App) -> AppResult {
    COUNT.update(deps.storage, |count| AppResult::Ok(count + 1))?;

    Ok(app.response("increment"))
}

fn reset(deps: DepsMut, info: MessageInfo, count: i32, app: App) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    COUNT.save(deps.storage, &count)?;

    Ok(app.response("reset"))
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: App) -> AppResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}
