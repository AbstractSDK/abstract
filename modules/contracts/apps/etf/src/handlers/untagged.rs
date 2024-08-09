use cosmwasm_std::{DepsMut, Env, MessageInfo};

use crate::{
    contract::{EtfApp, EtfResult},
    msg::MyUntaggedMsg,
};

use super::receive_cw20;

pub fn untagged_handler(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: EtfApp,
    untagged_msg: MyUntaggedMsg,
) -> EtfResult {
    match untagged_msg {
        MyUntaggedMsg::Receive(cw20_msg) => receive_cw20(deps, env, msg_info, module, cw20_msg),
    }
}
