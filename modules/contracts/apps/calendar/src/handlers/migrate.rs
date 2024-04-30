use abstract_app::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env};

use crate::{
    contract::{CalendarApp, CalendarAppResult},
    msg::CalendarMigrateMsg,
};

/// Handle the app migrate msg
/// The top-level Abstract app does version checking and dispatches to this handler
pub fn migrate_handler(
    _deps: DepsMut,
    _env: Env,
    app: CalendarApp,
    _msg: CalendarMigrateMsg,
) -> CalendarAppResult {
    Ok(app.response("migrate"))
}
