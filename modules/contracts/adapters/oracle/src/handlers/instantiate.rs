use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response};

use crate::contract::{OracleAdapter, OracleResult};

pub fn instantiate_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _module: OracleAdapter,
    _msg: Empty,
) -> OracleResult {
    Ok(Response::default())
}
