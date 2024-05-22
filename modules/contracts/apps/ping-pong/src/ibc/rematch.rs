use abstract_app::objects::chain_name::ChainName;
use abstract_app::sdk::IbcInterface;
use abstract_app::std::ibc::{CallbackInfo, CallbackResult};
use abstract_app::std::proxy;
use abstract_app::{sdk::AbstractResponse, std::ibc::IbcResponseMsg};
use cosmwasm_std::{from_json, to_json_binary, DepsMut, Env, MessageInfo, StdError, WasmQuery};

use crate::contract::{App, AppResult};
use crate::msg::{AppQueryMsg, PingPongIbcMsg, PreviousPingPongResponse};
use crate::state::PREVIOUS_PING_PONG;

use super::{PING_CALLBACK, REMOTE_PREVIOUS_PING_PONG_CALLBACK};

// TODO: this method is not consistent and was used just for testing
// We need module to module ibc query if we want to support this type of queries
pub fn proxy_config(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: App,
    response: IbcResponseMsg,
) -> AppResult {
    let msg = match response.result {
        CallbackResult::Query { query: _, result } => {
            let res_bin = result.unwrap().pop().unwrap();
            let proxy::ConfigResponse { modules } = from_json(res_bin)?;
            let remote_ping_pong_addr = modules[1].clone();
            let ibc_client = app.ibc_client(deps.as_ref());
            ibc_client.ibc_query(
                from_json(response.msg.clone().unwrap())?,
                WasmQuery::Smart {
                    contract_addr: remote_ping_pong_addr,
                    msg: to_json_binary(&crate::msg::QueryMsg::Module(
                        AppQueryMsg::PreviousPingPong {},
                    ))?,
                },
                CallbackInfo {
                    id: REMOTE_PREVIOUS_PING_PONG_CALLBACK.to_owned(),
                    msg: response.msg,
                },
            )?
        }
        CallbackResult::FatalError(_) => todo!(),
        // It was query, can't be execute
        CallbackResult::Execute { .. } => {
            unreachable!()
        }
    };

    Ok(app.response("proxy_config_callback").add_message(msg))
}

pub fn rematch_ping_pong(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    app: App,
    response: IbcResponseMsg,
) -> AppResult {
    let (msg, pongs) = match response.result {
        CallbackResult::Query { query: _, result } => {
            let res_bin = result.unwrap().pop().unwrap();
            let PreviousPingPongResponse { pongs, host_chain } = from_json(res_bin)?;
            let current_chain_name = ChainName::new(&env);
            if host_chain.is_some_and(|host_chain| current_chain_name == host_chain) {
                let host_chain: ChainName = from_json(response.msg.clone().unwrap())?;
                let pongs = pongs.unwrap();
                PREVIOUS_PING_PONG.save(deps.storage, &(pongs, host_chain.clone()))?;

                let ibc_client = app.ibc_client(deps.as_ref());
                (
                    ibc_client.module_ibc_action(
                        host_chain,
                        app.module_info()?,
                        &PingPongIbcMsg { pongs },
                        Some(CallbackInfo::new(PING_CALLBACK.to_owned(), None)),
                    )?,
                    pongs,
                )
            } else {
                return Err(StdError::generic_err("No rematch").into());
            }
        }
        CallbackResult::FatalError(_) => todo!(),
        // It was query, can't be execute
        CallbackResult::Execute { .. } => {
            unreachable!()
        }
    };

    Ok(app
        .response("rematch_ping_pong")
        .add_attribute("pongs_left", pongs.to_string())
        .add_message(msg))
}
