use crate::contract::{CalendarApp, CalendarAppResult};
use crate::msg::AppMigrateMsg;
use abstract_sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Response};

/// Handle the app migrate msg
/// The top-level Abstract app does version checking and dispatches to this handler
pub fn migrate_handler(
    _deps: DepsMut,
    _env: Env,
    app: CalendarApp,
    _msg: AppMigrateMsg,
) -> CalendarAppResult {
    Ok(app.tag_response(Response::default(), "migrate"))
}
