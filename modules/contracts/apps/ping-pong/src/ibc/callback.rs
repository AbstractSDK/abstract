use crate::{
    contract::{App, AppResult},
    error::AppError,
    msg::PreviousPingPongResponse,
    state::PREVIOUS_PING_PONG,
};

use abstract_app::{
    objects::chain_name::ChainName,
    std::ibc::{Callback, IbcResult},
};
use cosmwasm_std::{from_json, DepsMut, Env, MessageInfo};

use super::PingPongIbcCallbacks;

pub fn ibc_callback(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    app: App,
    callback: Callback,
    result: IbcResult,
) -> AppResult {
    match from_json(callback.msg)? {
        PingPongIbcCallbacks::Rematch { rematch_chain } => {
            rematch_callback(deps, env, app, result, rematch_chain)
        }
    }
}

pub fn rematch_callback(
    deps: DepsMut,
    env: Env,
    app: App,
    result: IbcResult,
    rematch_chain: ChainName,
) -> AppResult {
    let (_, result) = result.get_query_result(0)?;
    let PreviousPingPongResponse { pongs, host_chain } = from_json(result)?;
    if host_chain.map_or(false, |host| host == ChainName::new(&env)) {
        let pongs = pongs.unwrap();
        PREVIOUS_PING_PONG.save(deps.storage, &(pongs, rematch_chain.clone()))?;
        crate::handlers::execute::_ping_pong(deps, pongs, rematch_chain, app)
    } else {
        Err(AppError::NothingToRematch {})
    }
}
