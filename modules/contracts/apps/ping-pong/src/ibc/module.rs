use abstract_app::{sdk::AbstractResponse, std::ibc::ModuleIbcInfo};
use cosmwasm_std::{ensure, ensure_eq, from_json, Binary, DepsMut, Env, Response};

use crate::{
    contract::{App, AppResult},
    error::AppError,
    msg::{PingOrPong, PingPongIbcMsg},
};

pub fn receive_module_ibc(
    deps: DepsMut,
    _env: Env,
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
    let mut ping_msg: PingPongIbcMsg = from_json(msg)?;

    ensure!(matches!(ping_msg.hand, PingOrPong::Ping), AppError::FirstPlayMustBePing {});

    // Respond with Pong in Ack

    ping_msg.pongs -= 1;
    if ping_msg.pongs > 0 {
        crate::handlers::execute::_ping_pong(deps, ping_msg.pongs, source_module.chain, app)
    } else {
        // Done ping-ponging
        CURRENT_PONGS.save(deps.storage, &ping_msg.pongs)?;
        Ok(app.response("ping_ponged"))
    }
}
