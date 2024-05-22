use abstract_app::std::ibc::CallbackResult;
use abstract_app::{sdk::AbstractResponse, std::ibc::IbcResponseMsg};
use cosmwasm_std::{from_json, DepsMut, Env, MessageInfo};

use crate::{
    contract::{App, AppResult},
    state::CURRENT_PONGS,
};

pub fn ping_callback(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: App,
    response: IbcResponseMsg,
) -> AppResult {
    let is_error = match response.result {
        CallbackResult::Execute {
            initiator_msg,
            result,
        } => {
            // Need to clean state in case we sent last pong
            let ibc_pong_msg: crate::msg::PingPongIbcMsg = from_json(&initiator_msg)?;
            if ibc_pong_msg.pongs == 1 {
                CURRENT_PONGS.save(deps.storage, &0)?;
            }
            result.is_err()
        }
        CallbackResult::FatalError(_) => true,
        // It was execute, can't be query
        CallbackResult::Query { .. } => unreachable!(),
    };

    if is_error {
        // Need to clean state if tx failed
        CURRENT_PONGS.save(deps.storage, &0)?;
        Ok(adapter
            .response("ping_pong_failed")
            .add_attribute("pongs_left", "0"))
    } else {
        Ok(adapter.response("ping_callback"))
    }
}
