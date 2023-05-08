use crate::contract::{TemplateApp, TemplateResult};
use crate::msg::{ConfigResponse, TemplateQueryMsg};
use crate::state::CONFIG;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _etf: &TemplateApp,
    msg: TemplateQueryMsg,
) -> TemplateResult<Binary> {
    match msg {
        TemplateQueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}
