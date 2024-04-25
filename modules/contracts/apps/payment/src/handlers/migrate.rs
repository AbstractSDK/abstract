use abstract_app::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env};

use crate::{
    contract::{AppResult, PaymentApp},
    msg::AppMigrateMsg,
};

/// Handle the app migrate msg
/// The top-level Abstract app does version checking and dispatches to this handler
pub fn migrate_handler(
    _deps: DepsMut,
    _env: Env,
    app: PaymentApp,
    _msg: AppMigrateMsg,
) -> AppResult {
    Ok(app.response("migrate"))
}
