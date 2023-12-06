use crate::contract::{AppResult, DCAApp};
use crate::msg::{ConfigResponse, DCAQueryMsg, DCAResponse};
use crate::state::{DCAId, CONFIG, DCA_LIST};
use abstract_core::objects::DexAssetPairing;
use abstract_sdk::features::AbstractNameService;
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
    let native_asset = app.name_service(deps).query(&asset)?;
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
        let name_service = app.name_service(deps);

        name_service.query(&DexAssetPairing::new(
            entry.source_asset.name.clone(),
            entry.target_asset.clone(),
            &entry.dex,
        ))?
    } else {
        vec![]
    };
    Ok(DCAResponse {
        dca,
        pool_references,
    })
}
