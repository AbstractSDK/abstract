use abstract_app::objects::chain_name::ChainName;
use abstract_app::objects::module::ModuleInfo;
use abstract_app::sdk::IbcInterface;
use abstract_app::std::ibc_client;
use abstract_app::traits::AbstractResponse;
use cosmwasm_std::{to_json_binary, wasm_execute, DepsMut, Env, MessageInfo};

use crate::contract::{App, AppResult};

use crate::msg::{AppExecuteMsg, PingPongIbcMsg};
use crate::state::{CONFIG, CURRENT_PONGS};

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::PingPong {
            pongs,
            remote_chain,
        } => ping_pong(deps, info, pongs, remote_chain, app),
    }
}

fn ping_pong(
    deps: DepsMut,
    info: MessageInfo,
    pongs: u32,
    remote_chain: ChainName,
    app: App,
) -> AppResult {
    CURRENT_PONGS.save(deps.storage, &pongs)?;

    let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;
    let msg = ibc_client::ExecuteMsg::ModuleIbcAction {
        host_chain: remote_chain.to_string(),
        target_module: current_module_info,
        msg: to_json_binary(&PingPongIbcMsg { pongs })?,
        callback_info: None,
    };
    let ibc_client_addr = app.ibc_client(deps.as_ref()).module_address()?;
    let ibc_msg = wasm_execute(ibc_client_addr, &msg, vec![])?;

    Ok(app.response("ping_pong").add_message(ibc_msg))
}
