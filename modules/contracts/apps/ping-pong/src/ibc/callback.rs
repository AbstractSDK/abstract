use abstract_app::{
    objects::chain_name::ChainName,
    sdk::AbstractResponse,
    std::ibc::{Callback, IbcResult},
};
use cosmwasm_std::{from_json, DepsMut, Env, MessageInfo};

use crate::{
    contract::{App, AppResult},
    handlers::execute::ping_pong,
    msg::{BlockHeightResponse, PingPongCallbackMsg},
    state::{LOSSES, WINS},
};

pub fn ibc_callback(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    app: App,
    callback: Callback,
    result: IbcResult,
) -> AppResult {
    match from_json(callback.msg)? {
        PingPongCallbackMsg::Pinged { opponent_chain } => {
            // TODO: use response data here in the future
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
                    return ping_pong(deps, opponent_chain, app);
                }
                // we lost
                LOSSES.update(deps.storage, |l| AppResult::Ok(l + 1))?;
                Ok(app.response("lost"))
            } else {
                WINS.update(deps.storage, |w| AppResult::Ok(w + 1))?;
                Ok(app.response("won"))
            }
        }
        PingPongCallbackMsg::QueryBlockHeight { opponent_chain } => {
            play_if_win(deps, app, result, opponent_chain)
        }
    }
}

/// Play against the opponent if the block height is uneven (meaning we should win).
///
/// **Note**: The block height of the opponent chain changes all the time so we can't actually predict that we will win! This is just for demo purposes.
pub fn play_if_win(
    deps: DepsMut,
    app: App,
    result: IbcResult,
    opponent_chain: ChainName,
) -> AppResult {
    let (_, result) = result.get_query_result(0)?;
    let BlockHeightResponse { height } = from_json(result)?;

    // If uneven we play
    if height % 2 == 1 {
        ping_pong(deps, opponent_chain, app)
    } else {
        Ok(app.response("dont_play"))
    }
}
