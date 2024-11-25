use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo};

use crate::contract::{OracleAdapter, OracleResult};

pub fn execute_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _module: OracleAdapter,
    _msg: Empty,
) -> OracleResult {
    unimplemented!("No execution for this adapter")
}
