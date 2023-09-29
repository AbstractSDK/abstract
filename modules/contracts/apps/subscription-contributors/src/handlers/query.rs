use crate::contract::{App, AppResult};
use crate::msg::{AppQueryMsg, ConfigResponse};
use crate::state::CONTRIBUTION_CONFIG;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

pub fn query_handler(deps: Deps, _env: Env, _app: &App, msg: AppQueryMsg) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_binary(&query_config(deps)?),
        AppQueryMsg::State {} => todo!(),
        AppQueryMsg::ContributorState { os_id } => todo!(),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONTRIBUTION_CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}
