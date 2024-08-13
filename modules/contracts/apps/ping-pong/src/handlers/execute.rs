use abstract_app::{
    objects::TruncatedChainId,
    sdk::{IbcClient, IbcInterface},
    std::{ibc::Callback, ibc_client::InstalledModuleIdentification},
    traits::{AbstractResponse, AccountIdentification},
};
use cosmwasm_std::{CosmosMsg, DepsMut, Env, MessageInfo};

use crate::{
    contract::{App, AppResult},
    msg::{AppExecuteMsg, AppQueryMsg, PingOrPong, PingPongCallbackMsg, PingPongIbcMsg},
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    module: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::PingPong { opponent_chain } => ping_pong(deps, opponent_chain, module),
        AppExecuteMsg::QueryAndMaybePingPong {
            opponent_chain: host_chain,
        } => query_and_ping(&env, deps, host_chain, module),
    }
}

pub(crate) fn ping_pong(deps: DepsMut, opponent_chain: TruncatedChainId, module: App) -> AppResult {
    // # ANCHOR: ibc_client
    let self_module_info = module.module_info()?;
    let ibc_client: IbcClient<_> = module.ibc_client(deps.as_ref());
    let ibc_action: CosmosMsg = ibc_client.module_ibc_action(
        opponent_chain.clone(),
        self_module_info,
        // Start by playing a Ping
        &PingPongIbcMsg {
            hand: PingOrPong::Ping,
        },
        Some(Callback::new(&PingPongCallbackMsg::Pinged {
            opponent_chain,
        })?),
    )?;
    // # ANCHOR_END: ibc_client

    Ok(module
        .response("ping_pong")
        .add_attribute("play", "ping")
        .add_message(ibc_action))
}

fn query_and_ping(
    env: &Env,
    deps: DepsMut,
    opponent_chain: TruncatedChainId,
    module: App,
) -> AppResult {
    let ibc_client = app.ibc_client(deps.as_ref());
    let remote_account_id = app
        .account_id(deps.as_ref())?
        .into_remote_account_id(TruncatedChainId::new(env), opponent_chain.clone());

    let module_query = ibc_client.module_ibc_query(
        opponent_chain.clone(),
        InstalledModuleIdentification {
            module_info: app.module_info()?,
            account_id: Some(remote_account_id),
        },
        &crate::msg::QueryMsg::from(AppQueryMsg::BlockHeight {}),
        Callback::new(&PingPongCallbackMsg::QueryBlockHeight { opponent_chain })?,
    )?;

    Ok(module.response("rematch").add_message(module_query))
}
