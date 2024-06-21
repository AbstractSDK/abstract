use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

use crate::{
    contract::{App, AppResult},
    msg::{AppQueryMsg, BlockHeightResponse, WinsResponse},
    state::{LOSSES, WINS},
};

pub fn query_handler(deps: Deps, env: Env, _app: &App, msg: AppQueryMsg) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Wins {} => to_json_binary(&query_wins(deps)?),
        AppQueryMsg::BlockHeight {} => to_json_binary(&query_block_height(env)?),
    }
    .map_err(Into::into)
}

fn query_wins(deps: Deps) -> StdResult<WinsResponse> {
    let wins = WINS.load(deps.storage)?;
    let losses = LOSSES.load(deps.storage)?;

    Ok(WinsResponse { wins, losses })
}

fn query_block_height(env: Env) -> StdResult<BlockHeightResponse> {
    Ok(BlockHeightResponse {
        height: env.block.height,
    })
}
