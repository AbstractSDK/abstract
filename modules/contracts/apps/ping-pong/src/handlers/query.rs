use crate::contract::{App, AppResult};
use crate::msg::{AppQueryMsg, ConfigResponse, PongsResponse};
use crate::state::{CONFIG, CURRENT_PONGS};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

pub fn query_handler(deps: Deps, _env: Env, _app: &App, msg: AppQueryMsg) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        AppQueryMsg::Pongs {} => to_json_binary(&query_pongs(deps)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}

fn query_pongs(deps: Deps) -> StdResult<PongsResponse> {
    let pongs = CURRENT_PONGS.load(deps.storage)?;
    Ok(PongsResponse { pongs })
}
