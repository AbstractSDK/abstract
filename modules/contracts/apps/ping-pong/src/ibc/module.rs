use abstract_app::{sdk::AbstractResponse, std::ibc::ModuleIbcInfo};
use cosmwasm_std::{ensure, ensure_eq, from_json, Binary, DepsMut, Env, Response};

use crate::{
    contract::{App, AppResult},
    error::AppError,
    msg::{PingOrPong, PingPongIbcMsg},
    state::LOSSES,
};

// # ANCHOR: module_ibc
pub fn receive_module_ibc(
    deps: DepsMut,
    env: Env,
    app: App,
    source_module: ModuleIbcInfo,
    msg: Binary,
) -> AppResult<Response> {
    let this_module_info = app.module_info()?;
    ensure_eq!(
        source_module.module,
        this_module_info,
        AppError::NotPingPong {
            source_module: source_module.module.clone()
        }
    );
    let ping_msg: PingPongIbcMsg = from_json(msg)?;
// # ANCHOR_END: module_ibc
    ensure!(
        matches!(ping_msg.hand, PingOrPong::Ping),
        AppError::FirstPlayMustBePing {}
    );

    // Respond with Pong in Ack if
    let mut resp = app.response("ping_ponged");

    // if block is even, return pong
    let is_even = env.block.height % 2 == 0;
    if is_even {
        // TODO: return `PingOrPong::Pong` in response.data instead of event.
        resp = resp.add_attribute("play", "pong");
    } else {
        // else we lost
        LOSSES.update(deps.storage, |l| AppResult::Ok(l + 1))?;
    }
    Ok(resp)
}
