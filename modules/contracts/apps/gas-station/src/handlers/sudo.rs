use cosmwasm_std::{Coin, DepsMut, Env, Response};

use crate::contract::{AppResult, GasStationApp};
use crate::msg::GasStationSudoMsg;

pub fn sudo_handler(
    deps: DepsMut,
    _env: Env,
    app: GasStationApp,
    msg: GasStationSudoMsg,
) -> AppResult {
    match msg {
        GasStationSudoMsg::BlockBeforeSend {
            from,
            to,
            amount,
        } => before_send_hook(deps, app, from, to, amount),
    }
}

pub fn before_send_hook(
    deps: DepsMut,
    app: GasStationApp,
    from: String,
    to: String,
    amount: Coin,
) -> AppResult {
    // TODO: revoke permissions

    Ok(Response::new().add_attribute("action", "before_send"))
}
