use crate::contract::{CroncatApp, CroncatResult};

use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply, Response};

pub fn instantiate_reply(
    _deps: DepsMut,
    _env: Env,
    app: CroncatApp,
    _reply: Reply,
) -> CroncatResult {
    Ok(app.tag_response(Response::default(), "instantiate_reply"))
}
