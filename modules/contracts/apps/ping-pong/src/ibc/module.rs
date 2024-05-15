use abstract_app::{
    objects::module::ModuleInfo,
    sdk::{AbstractResponse, IbcInterface},
    std::ibc::ModuleIbcMsg,
};
use cosmwasm_std::{from_json, DepsMut, Env, Response};

use crate::{
    contract::{App, AppResult},
    msg::PingPongIbcMsg,
    state::CURRENT_PONGS,
};

pub fn receive_module_ibc(
    deps: DepsMut,
    _env: Env,
    app: App,
    msg: ModuleIbcMsg,
) -> AppResult<Response> {
    let mut ping_msg: PingPongIbcMsg = from_json(&msg.msg)?;

    CURRENT_PONGS.save(deps.storage, &ping_msg.pongs)?;
    if ping_msg.pongs > 0 {
        let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;
        let ibc_client = app.ibc_client(deps.as_ref());
        ping_msg.pongs -= 1;
        let ibc_action =
            ibc_client.module_ibc_action(msg.client_chain, current_module_info, &ping_msg, None)?;
        Ok(app.response("ping_back").add_message(ibc_action))
    } else {
        // Done ping-ponging
        Ok(app.response("ping_ponged"))
    }
}
