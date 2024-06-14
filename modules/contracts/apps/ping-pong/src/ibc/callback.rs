use crate::{
    contract::{App, AppResult},
    state::CURRENT_PONGS,
};

use abstract_app::{
    sdk::AbstractResponse,
    std::ibc::{Callback, IbcResult},
};
use cosmwasm_std::{from_json, DepsMut, Env, MessageInfo};

use super::PingPongCallbacks;

pub fn ping_callback(deps: DepsMut, app: App, result: IbcResult) -> AppResult {
    let is_error = match result {
        IbcResult::Execute {
            initiator_msg,
            result,
        } => {
            // Need to clean state in case we sent last pong
            let ibc_pong_msg: crate::msg::PingPongIbcMsg = from_json(initiator_msg)?;
            if ibc_pong_msg.pongs == 1 {
                CURRENT_PONGS.save(deps.storage, &0)?;
            }
            result.is_err()
        }
        IbcResult::FatalError(_) => true,
        // It was execute, can't be query
        IbcResult::Query { .. } => unreachable!(),
    };

    if is_error {
        // Need to clean state if tx failed
        CURRENT_PONGS.save(deps.storage, &0)?;
        Ok(app
            .response("ping_pong_failed")
            .add_attribute("pongs_left", "0"))
    } else {
        Ok(app.response("ping_callback"))
    }
}

pub fn ibc_callback(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: App,
    callback: Callback,
    result: IbcResult,
) -> AppResult {
    match from_json(&callback.msg)? {
        PingPongCallbacks::Ping => ping_callback(deps, app, result),
        _ => unreachable!(),
    }
}
