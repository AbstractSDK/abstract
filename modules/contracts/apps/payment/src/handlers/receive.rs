use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw20::Cw20ReceiveMsg;
use cw_asset::Asset;

use crate::contract::{AppResult, PaymentApp};

pub fn receive_handler(
    deps: DepsMut,
    _env: Env,
    mut info: MessageInfo,
    app: PaymentApp,
    msg: Cw20ReceiveMsg,
) -> AppResult {
    let Cw20ReceiveMsg {
        sender,
        amount,
        msg: _,
    } = msg;

    let receipt = Asset::cw20(info.sender, amount);

    info.sender = deps.api.addr_validate(&sender)?;
    crate::handlers::execute::tip(deps, info, app, Some(receipt))
}
