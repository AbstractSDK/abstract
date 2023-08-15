use cosmwasm_std::{Coin, DepsMut, Env, Response};

use crate::contract::{GasStationResult, GasStationApp};
use crate::msg::GasStationSudoMsg;

pub fn sudo_handler(
    deps: DepsMut,
    _env: Env,
    app: GasStationApp,
    msg: GasStationSudoMsg,
) -> GasStationResult {
    match msg {
        GasStationSudoMsg::BlockBeforeSend { from, to, amount } => {
            before_send_hook(deps, app, from, to, amount)
        }
    }
}

pub fn before_send_hook(
    _deps: DepsMut,
    _app: GasStationApp,
    _from: String,
    _to: String,
    _amount: Coin,
) -> GasStationResult {
    // TODO: revoke permissions

    Ok(Response::new().add_attribute("action", "before_send"))
}
