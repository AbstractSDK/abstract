use abstract_app::{
    objects::TruncatedChainId,
    sdk::AbstractResponse,
    std::ibc::{Callback, IbcResult},
};
use cosmwasm_std::{from_json, DepsMut, Env};

use crate::{
    contract::{App, AppResult},
    handlers::execute::ping_pong,
    msg::{BlockHeightResponse, PingPongCallbackMsg},
    state::{LOSSES, WINS},
};

pub fn ibc_callback(
    deps: DepsMut,
    env: Env,
    module: App,
    callback: Callback,
    result: IbcResult,
) -> AppResult {
    match from_json(callback.msg)? {
        PingPongCallbackMsg::Pinged { opponent_chain } => {
            let exec_events = result.get_execute_events()?;

            let pong = exec_events.into_iter().find(|e| {
                e.ty == "wasm"
                    && e.attributes
                        .iter()
                        .any(|a| a.key == "play" && a.value == "pong")
            });
            if pong.is_some() {
                // if block is even, return ping
                let is_even = env.block.height % 2 == 0;
                if is_even {
                    // We play ping again
                    return ping_pong(deps, opponent_chain, module);
                }
                // we lost
                LOSSES.update(deps.storage, |l| AppResult::Ok(l + 1))?;
                Ok(module.response("lost"))
            } else {
                WINS.update(deps.storage, |w| AppResult::Ok(w + 1))?;
                Ok(module.response("won"))
            }
        }
        PingPongCallbackMsg::QueryBlockHeight { opponent_chain } => {
            play_if_win(deps, module, result, opponent_chain)
        }
    }
}

/// Play against the opponent if the block height is uneven (meaning we should win).
///
/// **Note**: The block height of the opponent chain changes all the time so we can't actually predict that we will win! This is just for demo purposes.
pub fn play_if_win(
    deps: DepsMut,
    module: App,
    result: IbcResult,
    opponent_chain: TruncatedChainId,
) -> AppResult {
    let (_, result) = result.get_query_result(0)?;
    let BlockHeightResponse { height } = from_json(result)?;

    // If uneven we play
    if height % 2 == 1 {
        ping_pong(deps, opponent_chain, module)
    } else {
        Ok(module.response("dont_play"))
    }
}
