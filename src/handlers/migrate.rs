use crate::contract::{TemplateApp, TemplateResult};
use crate::msg::TemplateMigrateMsg;
use abstract_sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Response};

/// Handle the app migrate msg
/// The top-level Abstract app does version checking and dispatches to this handler
pub fn migrate_handler(
    _deps: DepsMut,
    _env: Env,
    app: TemplateApp,
    _msg: TemplateMigrateMsg,
) -> TemplateResult {
    Ok(app.tag_response(Response::default(), "migrate"))
}
