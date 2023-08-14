use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{AppResult, GasStationApp};
use crate::msg::AppInstantiateMsg;

pub fn instantiate_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: GasStationApp,
    _msg: AppInstantiateMsg,
) -> AppResult {
    Ok(Response::new())
}
