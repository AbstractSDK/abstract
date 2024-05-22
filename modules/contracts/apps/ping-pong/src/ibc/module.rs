use abstract_app::{
    sdk::{AbstractResponse, IbcInterface},
    std::ibc::{CallbackInfo, ModuleIbcMsg},
};
use cosmwasm_std::{ensure_eq, from_json, DepsMut, Env, Response};

use crate::{
    contract::{App, AppResult},
    error::AppError,
    ibc::PING_CALLBACK,
    msg::PingPongIbcMsg,
    state::CURRENT_PONGS,
};

pub fn receive_module_ibc(
    deps: DepsMut,
    _env: Env,
    app: App,
    msg: ModuleIbcMsg,
) -> AppResult<Response> {
    let current_module_info = app.module_info()?;
    ensure_eq!(
        msg.source_module,
        current_module_info,
        AppError::NotPingPong {
            source_module: msg.source_module.clone()
        }
    );
    let mut ping_msg: PingPongIbcMsg = from_json(&msg.msg)?;

    ping_msg.pongs = ping_msg.pongs.saturating_sub(1);
    CURRENT_PONGS.save(deps.storage, &ping_msg.pongs)?;
    if ping_msg.pongs > 0 {
        let ibc_client = app.ibc_client(deps.as_ref());
        let ibc_action = ibc_client.module_ibc_action(
            msg.client_chain,
            current_module_info,
            &ping_msg,
            Some(CallbackInfo::new(PING_CALLBACK.to_owned(), None)),
        )?;
        Ok(app
            .response("ping_back")
            .add_attribute("pongs_left", ping_msg.pongs.to_string())
            .add_message(ibc_action))
    } else {
        // Done ping-ponging
        Ok(app.response("ping_ponged"))
    }
}
