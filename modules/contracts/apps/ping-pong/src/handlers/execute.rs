use abstract_app::{
    objects::TruncatedChainId,
    sdk::{IbcClient, IbcInterface},
    std::{ibc::Callback, ibc_client::InstalledModuleIdentification},
    traits::{AbstractResponse, AccountIdentification},
};
use cosmwasm_std::{Coin, CosmosMsg, DepsMut, Env, MessageInfo};

use crate::{
    contract::{App, AppResult},
    msg::{AppExecuteMsg, AppQueryMsg, PingOrPong, PingPongCallbackMsg, PingPongIbcMsg},
};

use super::ICS20_CALLBACK_ID;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    module: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::PingPong { opponent_chain } => ping_pong(deps, &env, opponent_chain, module),
        AppExecuteMsg::QueryAndMaybePingPong {
            opponent_chain: host_chain,
        } => query_and_ping(deps, &env, host_chain, module),
        AppExecuteMsg::FundOpponent {
            opponent_chain,
            funds,
            callback,
        } => fund_opponent(deps, &env, opponent_chain, funds, callback, module),
    }
}

pub(crate) fn ping_pong(
    deps: DepsMut,
    env: &Env,
    opponent_chain: TruncatedChainId,
    module: App,
) -> AppResult {
    // # ANCHOR: ibc_client
    let self_module_info = module.module_info()?;
    let ibc_client: IbcClient<_> = module.ibc_client(deps.as_ref(), env);
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
    deps: DepsMut,
    env: &Env,
    opponent_chain: TruncatedChainId,
    module: App,
) -> AppResult {
    let ibc_client = module.ibc_client(deps.as_ref(), env);
    let remote_account_id = module
        .account_id(deps.as_ref())?
        .into_remote_account_id(TruncatedChainId::new(env), opponent_chain.clone());

    let module_query = ibc_client.module_ibc_query(
        opponent_chain.clone(),
        InstalledModuleIdentification {
            module_info: module.module_info()?,
            account_id: Some(remote_account_id),
        },
        &crate::msg::QueryMsg::from(AppQueryMsg::BlockHeight {}),
        Callback::new(&PingPongCallbackMsg::QueryBlockHeight { opponent_chain })?,
    )?;

    Ok(module.response("rematch").add_message(module_query))
}

pub(crate) fn fund_opponent(
    deps: DepsMut,
    env: &Env,
    opponent_chain: TruncatedChainId,
    funds: Coin,
    callback: Callback,
    module: App,
) -> AppResult {
    let ibc_client: IbcClient<_> = module.ibc_client(deps.as_ref(), env);
    let msg =
        ibc_client.send_funds_with_callback(opponent_chain, funds, callback, ICS20_CALLBACK_ID)?;

    Ok(module.response("fund_opponent").add_submessage(msg))
}
