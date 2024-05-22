use crate::contract::{App, AppResult};
use crate::msg::{AppQueryMsg, PongsResponse, PreviousPingPongResponse};
use crate::state::{CURRENT_PONGS, PREVIOUS_PING_PONG};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

pub fn query_handler(deps: Deps, _env: Env, _app: &App, msg: AppQueryMsg) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Pongs {} => to_json_binary(&query_pongs(deps)?),
        AppQueryMsg::PreviousPingPong {} => to_json_binary(&query_previous_ping_pongs(deps)?),
    }
    .map_err(Into::into)
}

fn query_pongs(deps: Deps) -> StdResult<PongsResponse> {
    let pongs = CURRENT_PONGS.load(deps.storage)?;
    Ok(PongsResponse { pongs })
}

fn query_previous_ping_pongs(deps: Deps) -> StdResult<PreviousPingPongResponse> {
    let (pongs, host_chain) = PREVIOUS_PING_PONG.may_load(deps.storage)?.unzip();

    Ok(PreviousPingPongResponse { pongs, host_chain })
}
