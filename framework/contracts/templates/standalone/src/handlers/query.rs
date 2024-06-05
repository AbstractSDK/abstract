use crate::{
    msg::{ConfigResponse, CountResponse, MyStandaloneQueryMsg},
    state::{CONFIG, COUNT},
    MY_STANDALONE,
};

use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: MyStandaloneQueryMsg) -> StdResult<Binary> {
    let _standalone = &MY_STANDALONE;
    match msg {
        MyStandaloneQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        MyStandaloneQueryMsg::Count {} => to_json_binary(&query_count(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let count = COUNT.load(deps.storage)?;
    Ok(CountResponse { count })
}
