use crate::contract::{App, AppResult};
use crate::msg::AppMigrateMsg;
use abstract_sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Response};

/// Handle the app migrate msg
/// The top-level Abstract app does version checking and dispatches to this handler
pub fn migrate_handler(_deps: DepsMut, _env: Env, app: App, _msg: AppMigrateMsg) -> AppResult {
    Ok(app.tag_response(Response::default(), "migrate"))
}
