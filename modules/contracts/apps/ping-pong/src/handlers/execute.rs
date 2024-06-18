use abstract_app::objects::chain_name::ChainName;

use abstract_app::sdk::IbcInterface;
use abstract_app::std::ibc::Callback;
use abstract_app::std::ibc_client::InstalledModuleIdentification;
use abstract_app::traits::AbstractResponse;
use abstract_app::traits::AccountIdentification;
use cosmwasm_std::{DepsMut, Env, MessageInfo};

use crate::contract::{App, AppResult};

use crate::msg::{AppExecuteMsg, PingPongCallbackMsg, PingPongIbcMsg};
use crate::msg::{AppQueryMsg, PingOrPong};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::PingPong { opponent_chain } => ping_pong(deps, opponent_chain, app),
        AppExecuteMsg::QueryAndMaybePingPong {
            opponent_chain: host_chain,
        } => query_and_ping(&env, deps, host_chain, app),
    }
}

pub(crate) fn ping_pong(deps: DepsMut, opponent_chain: ChainName, app: App) -> AppResult {
    let current_module_info = app.module_info()?;
    let ibc_client = app.ibc_client(deps.as_ref());
    let ibc_action = ibc_client.module_ibc_action(
        opponent_chain.clone(),
        current_module_info,
        // Start by playing a Ping
        &PingPongIbcMsg {
            hand: PingOrPong::Ping,
        },
        Some(Callback::new(&PingPongCallbackMsg::Pinged {
            opponent_chain,
        })?),
    )?;

    Ok(app
        .response("ping_pong")
        .add_attribute("play", "ping")
        .add_message(ibc_action))
}

fn query_and_ping(env: &Env, deps: DepsMut, opponent_chain: ChainName, app: App) -> AppResult {
    let ibc_client = app.ibc_client(deps.as_ref());
    let dest_account_id = app
        .account_id(deps.as_ref())?
        .into_dest_account_id(ChainName::new(env), opponent_chain.clone());

    let module_query = ibc_client.module_ibc_query(
        opponent_chain.clone(),
        InstalledModuleIdentification {
            module_info: app.module_info()?,
            account_id: Some(dest_account_id),
        },
        &crate::msg::QueryMsg::from(AppQueryMsg::BlockHeight {}),
        Callback::new(&PingPongCallbackMsg::QueryBlockHeight { opponent_chain })?,
    )?;

    Ok(app.response("rematch").add_message(module_query))
}
