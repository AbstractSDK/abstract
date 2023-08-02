use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{AppResult, DiceApp};
use crate::msg::DiceAppInstantiateMsg;

pub fn instantiate_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: DiceApp,
    _msg: DiceAppInstantiateMsg,
) -> AppResult {
    Ok(Response::new())
}
