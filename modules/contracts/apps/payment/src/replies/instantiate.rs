use crate::contract::{AppResult, PaymentApp};

use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply, Response};

pub fn _instantiate_reply(_deps: DepsMut, _env: Env, app: PaymentApp, _reply: Reply) -> AppResult {
    Ok(app.tag_response(Response::default(), "instantiate_reply"))
}
