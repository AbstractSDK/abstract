use crate::contract::{AppResult, DCAApp};
use crate::msg::{ConfigResponse, DCAQueryMsg, DCAResponse};
use crate::state::{DCAId, CONFIG, DCA_LIST};
use abstract_core::objects::DexAssetPairing;
use abstract_sdk::features::AbstractNameService;
use abstract_sdk::Resolve;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env};
use cw_asset::AssetInfo;

pub fn query_handler(deps: Deps, _env: Env, app: &DCAApp, msg: DCAQueryMsg) -> AppResult<Binary> {
    match msg {
        DCAQueryMsg::Config {} => to_json_binary(&query_config(deps, app)?),
        DCAQueryMsg::DCA { dca_id } => to_json_binary(&query_dca(deps, app, dca_id)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps, app: &DCAApp) -> AppResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let asset = AssetInfo::native(config.native_denom);
    let native_asset = app
        .ans_host(deps)?
        .query_asset_reverse(&deps.querier, &asset)?;
    Ok(ConfigResponse {
        native_asset,
        dca_creation_amount: config.dca_creation_amount,
        refill_threshold: config.refill_threshold,
        max_spread: config.max_spread,
    })
}

/// Get dca
fn query_dca(deps: Deps, app: &DCAApp, dca_id: DCAId) -> AppResult<DCAResponse> {
    let dca = DCA_LIST.may_load(deps.storage, dca_id)?;

    let pool_references = if let Some(entry) = dca.as_ref() {
        let ans_host = app.ans_host(deps)?;

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
