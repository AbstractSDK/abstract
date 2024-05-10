use abstract_app::{
    objects::module::ModuleInfo,
    sdk::{AbstractResponse, IbcInterface},
    std::{ibc::ModuleIbcMsg, ibc_client},
};
use cosmwasm_std::{from_json, to_json_binary, wasm_execute, DepsMut, Env, Response};

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
    let ping_msg: PingPongIbcMsg = from_json(&msg.msg)?;

    CURRENT_PONGS.save(deps.storage, &ping_msg.pongs)?;
    if ping_msg.pongs > 0 {
        let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;
        let msg = ibc_client::ExecuteMsg::ModuleIbcAction {
            host_chain: msg.client_chain.to_string(),
            target_module: current_module_info,
            msg: to_json_binary(&PingPongIbcMsg {
                pongs: ping_msg.pongs - 1,
            })?,
            callback_info: None,
        };
        let ibc_client_addr = app.ibc_client(deps.as_ref()).module_address()?;
        let ibc_msg = wasm_execute(ibc_client_addr, &msg, vec![])?;
        Ok(app.response("ping_back").add_message(ibc_msg))
    } else {
        Ok(app.response("ping_ponged"))
    }
}
