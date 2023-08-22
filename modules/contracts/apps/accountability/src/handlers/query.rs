use crate::contract::{AccApp, AppResult};
use crate::msg::{AccQueryMsg, AccResponse, ConfigResponse};
use crate::state::{ACC_LIST, CONFIG};
use abstract_core::objects::DexAssetPairing;
use abstract_sdk::features::AbstractNameService;
use abstract_sdk::Resolve;
use cosmwasm_std::{to_binary, Binary, Deps, Env};
use cw_asset::AssetInfo;

pub fn query_handler(deps: Deps, _env: Env, app: &AccApp, msg: AccQueryMsg) -> AppResult<Binary> {
    match msg {
        AccQueryMsg::Config {} => to_binary(&query_config(deps, app)?),
        AccQueryMsg::Acc { acc_id } => to_binary(&query_acc(deps, app, acc_id)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps, app: &AccApp) -> AppResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let asset = AssetInfo::native(config.native_denom);
    let native_asset = app
        .ans_host(deps)?
        .query_asset_reverse(&deps.querier, &asset)?;
    Ok(ConfigResponse {
        native_asset,
        forfeit_amount: config.forfeit_amount,
        refill_threshold: config.refill_threshold,
    })
}

/// Get accountability
fn query_accountability(deps: Deps, app: &AccApp, acc_id: String) -> AppResult<AccResponse> {
    let dca = ACC_LIST.may_load(deps.storage, acc_id)?;
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
    Ok(AccResponse {
        acc,
        pool_references,
    })
}
