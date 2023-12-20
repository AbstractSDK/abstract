use crate::contract::{CalendarApp, CalendarAppResult};
use crate::msg::CalendarMigrateMsg;
use abstract_sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env};

/// Handle the app migrate msg
/// The top-level Abstract app does version checking and dispatches to this handler
pub fn migrate_handler(
    _deps: DepsMut,
    _env: Env,
    app: CalendarApp,
    _msg: CalendarMigrateMsg,
) -> CalendarAppResult {
    Ok(app.tag_response("migrate"))
}
