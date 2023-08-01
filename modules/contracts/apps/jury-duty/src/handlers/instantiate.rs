use cosmwasm_std::{DepsMut, Env, MessageInfo};

use crate::contract::{AppResult, JuryDutyApp};
use crate::msg::JuryDutyInstantiateMsg;

pub fn instantiate_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: JuryDutyApp,
    _msg: JuryDutyInstantiateMsg,
) -> AppResult {
    Ok(cw3_flex_multisig::contract::instantiate(
        _deps, _env, _info, _msg,
    )?)
}
