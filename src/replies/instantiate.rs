use crate::contract::{AppResult, DCAApp};

use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply, Response};

pub fn instantiate_reply(_deps: DepsMut, _env: Env, app: DCAApp, _reply: Reply) -> AppResult {
    Ok(app.tag_response(Response::default(), "instantiate_reply"))
}
