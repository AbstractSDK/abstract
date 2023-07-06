use crate::contract::{AppResult, DCAApp};
use crate::msg::{ConfigResponse, DCAQueryMsg, DCAResponse};
use crate::state::{CONFIG, DCA_LIST};
use abstract_core::objects::DexAssetPairing;
use abstract_sdk::features::AbstractNameService;
use abstract_sdk::Resolve;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

pub fn query_handler(deps: Deps, _env: Env, app: &DCAApp, msg: DCAQueryMsg) -> AppResult<Binary> {
    match msg {
        DCAQueryMsg::Config {} => to_binary(&query_config(deps)?),
        DCAQueryMsg::DCA { dca_id } => to_binary(&query_dca(deps, app, dca_id)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

/// Get dca
fn query_dca(deps: Deps, app: &DCAApp, dca_id: String) -> AppResult<DCAResponse> {
    let dca = DCA_LIST.may_load(deps.storage, dca_id)?;
    let ans_host = app.ans_host(deps)?;
    let pool_references = if let Some(entry) = dca.as_ref() {
        DexAssetPairing::new(
            entry.source_asset.name.clone(),
            entry.target_asset.clone(),
            &entry.dex,
        )
        .resolve(&deps.querier, &ans_host)?
    } else {
        vec![]
    };
    Ok(DCAResponse {
        dca,
        pool_references,
    })
}
