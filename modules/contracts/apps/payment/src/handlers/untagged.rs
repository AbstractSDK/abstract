use cosmwasm_std::{DepsMut, Env, MessageInfo};

use crate::{
    contract::{AppResult, PaymentApp},
    msg::MyUntaggedMsg,
};

use super::receive_handler;

pub fn untagged_handler(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: PaymentApp,
    untagged_msg: MyUntaggedMsg,
) -> AppResult {
    match untagged_msg {
        MyUntaggedMsg::Receive(cw20_msg) => receive_handler(deps, env, msg_info, module, cw20_msg),
    }
}
