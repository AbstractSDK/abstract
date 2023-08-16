use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{GasStationApp, GasStationResult};
use crate::msg::GasStationInstantiateMsg;

pub fn instantiate_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: GasStationApp,
    _msg: GasStationInstantiateMsg,
) -> GasStationResult {
    Ok(Response::new())
}
