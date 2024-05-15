use abstract_app::objects::chain_name::ChainName;
use abstract_app::objects::module::ModuleInfo;
use abstract_app::sdk::IbcInterface;
use abstract_app::traits::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo};

use crate::contract::{App, AppResult};

use crate::msg::{AppExecuteMsg, PingPongIbcMsg};
use crate::state::CURRENT_PONGS;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::PingPong { pongs, host_chain } => {
            ping_pong(deps, info, pongs, host_chain, app)
        }
    }
}

fn ping_pong(
    deps: DepsMut,
    _info: MessageInfo,
    pongs: u32,
    host_chain: ChainName,
    app: App,
) -> AppResult {
    CURRENT_PONGS.save(deps.storage, &pongs)?;

    let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;
    let ibc_client = app.ibc_client(deps.as_ref());
    let ibc_action = ibc_client.module_ibc_action(
        host_chain,
        current_module_info,
        &PingPongIbcMsg { pongs },
        None,
    )?;

    Ok(app.response("ping_pong").add_message(ibc_action))
}
