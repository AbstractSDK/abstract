use crate::contract::{TemplateApp, TemplateResult};

use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply, Response};

pub fn instantiate_reply(
    _deps: DepsMut,
    _env: Env,
    app: TemplateApp,
    _reply: Reply,
) -> TemplateResult {
    Ok(app.tag_response(Response::default(), "instantiate_reply"))
}
