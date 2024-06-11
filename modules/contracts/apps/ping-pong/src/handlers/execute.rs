use abstract_app::objects::chain_name::ChainName;
use abstract_app::objects::module::ModuleInfo;
use abstract_app::sdk::IbcInterface;
use abstract_app::std::ibc::CallbackInfo;
use abstract_app::std::proxy;
use abstract_app::traits::AbstractResponse;
use cosmwasm_std::{to_json_binary, DepsMut, Env, MessageInfo, StdError, WasmQuery};

use crate::contract::{App, AppResult};

use crate::ibc::{PING_CALLBACK, QUERY_PROXY_CONFIG_CALLBACK};
use crate::msg::{AppExecuteMsg, PingPongIbcMsg};
use crate::state::{CURRENT_PONGS, PREVIOUS_PING_PONG};

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
        AppExecuteMsg::Rematch { host_chain } => rematch(deps, info, host_chain, app),
    }
}

fn ping_pong(
    deps: DepsMut,
    _info: MessageInfo,
    pongs: u32,
    host_chain: ChainName,
    app: App,
) -> AppResult {
    PREVIOUS_PING_PONG.save(deps.storage, &(pongs, host_chain.clone()))?;
    CURRENT_PONGS.save(deps.storage, &pongs)?;

    let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;
    let ibc_client = app.ibc_client(deps.as_ref());
    let ibc_action = ibc_client.module_ibc_action(
        host_chain,
        current_module_info,
        &PingPongIbcMsg { pongs },
        Some(CallbackInfo::new(PING_CALLBACK.to_owned(), None)),
    )?;

    Ok(app
        .response("ping_pong")
        .add_attribute("pongs_left", pongs.to_string())
        .add_message(ibc_action))
}

fn rematch(deps: DepsMut, _info: MessageInfo, host_chain: ChainName, app: App) -> AppResult {
    let ibc_client = app.ibc_client(deps.as_ref());
    let remote_proxy_addr = ibc_client
        .remote_proxy(&host_chain)?
        .ok_or(StdError::generic_err("remote proxy not found"))?;

    let ibc_query = ibc_client.ibc_query(
        host_chain.clone(),
        WasmQuery::Smart {
            contract_addr: remote_proxy_addr,
            msg: to_json_binary(&proxy::QueryMsg::Config {})?,
        },
        CallbackInfo::new(
            QUERY_PROXY_CONFIG_CALLBACK.to_owned(),
            Some(to_json_binary(&host_chain)?),
        ),
    )?;

    Ok(app.response("rematch").add_message(ibc_query))
}
