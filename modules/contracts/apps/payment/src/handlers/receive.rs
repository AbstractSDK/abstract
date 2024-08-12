use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw20::Cw20ReceiveMsg;
use cw_asset::Asset;

use crate::contract::{AppResult, PaymentApp};

// TODO: custom execute msg
pub fn receive_handler(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    module: PaymentApp,
    msg: Cw20ReceiveMsg,
) -> AppResult {
    let Cw20ReceiveMsg {
        sender,
        amount,
        msg: _,
    } = msg;

    let receipt = Asset::cw20(info.sender, amount);

    info.sender = deps.api.addr_validate(&sender)?;
    crate::handlers::execute::tip(deps, env, info, module, Some(receipt))
}
