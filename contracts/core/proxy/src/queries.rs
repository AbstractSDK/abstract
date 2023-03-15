use crate::contract::ProxyResult;

use abstract_os::objects::oracle::{AccountValue, Oracle};
use abstract_os::proxy::{
    AssetsConfigResponse, BaseAssetResponse, HoldingAmountResponse, OracleAsset,
};
use abstract_sdk::os::objects::AssetEntry;
use abstract_sdk::os::proxy::state::{ANS_HOST, STATE};
use abstract_sdk::os::proxy::{AssetsInfoResponse, ConfigResponse};
use abstract_sdk::Resolve;
use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};
use cw_asset::{Asset, AssetInfo};

const DEFAULT_LIMIT: u8 = 5;
const MAX_LIMIT: u8 = 20;

/// get the assets pricing information
pub fn query_oracle_asset_info(
    deps: Deps,
    last_asset: Option<AssetInfo>,
    limit: Option<u8>,
) -> ProxyResult<AssetsInfoResponse> {
    let oracle = Oracle::new();
    let assets = oracle.paged_asset_info(deps, last_asset, limit)?;
    Ok(AssetsInfoResponse {
        assets: assets
            .into_iter()
            .map(|(a, (p, c))| {
                (
                    a,
                    OracleAsset {
                        complexity: c,
                        price_source: p,
                    },
                )
            })
            .collect(),
    })
}

/// get the human-readable asset pricing information
pub fn query_oracle_asset_config(
    deps: Deps,
    last_asset: Option<AssetEntry>,
    limit: Option<u8>,
) -> ProxyResult<AssetsConfigResponse> {
    let oracle = Oracle::new();
    let assets = oracle.paged_asset_config(deps, last_asset, limit)?;
    Ok(AssetsConfigResponse { assets })
}

/// Returns the whitelisted modules
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = STATE.load(deps.storage)?;
    let modules: Vec<Addr> = state.modules;
    let resp = ConfigResponse {
        modules: modules
            .iter()
            .map(|module| -> String { module.to_string() })
            .collect(),
    };
    Ok(resp)
}

/// Returns the value of the amount of the specified asset
/// @param amount: The amount of the asset to compute the value of. If None, balance of the proxy account is used.
pub fn query_token_value(
    deps: Deps,
    env: Env,
    asset_entry: AssetEntry,
    amount: Option<Uint128>,
) -> ProxyResult<Uint128> {
    let oracle = Oracle::new();
    let ans_host = ANS_HOST.load(deps.storage)?;
    let asset_info = asset_entry.resolve(&deps.querier, &ans_host)?;
    let balance = amount.unwrap_or_else(|| {
        asset_info
            .query_balance(&deps.querier, env.contract.address)
            .unwrap()
    });
    let value = oracle.asset_value(deps, Asset::new(asset_info, balance))?;
    Ok(value)
}

/// Computes the total value locked in this contract
pub fn query_total_value(deps: Deps, env: Env) -> ProxyResult<AccountValue> {
    let mut oracle = Oracle::new();
    oracle
        .account_value(deps, &env.contract.address)
        .map_err(Into::into)
}

pub fn query_base_asset(deps: Deps) -> ProxyResult<BaseAssetResponse> {
    let oracle = Oracle::new();
    let base_asset = oracle.base_asset(deps)?;
    Ok(BaseAssetResponse { base_asset })
}

pub fn query_holding_amount(
    deps: Deps,
    env: Env,
    identifier: AssetEntry,
) -> ProxyResult<HoldingAmountResponse> {
    let ans_host = ANS_HOST.load(deps.storage)?;
    let asset_info = identifier.resolve(&deps.querier, &ans_host)?;
    Ok(HoldingAmountResponse {
        amount: asset_info.query_balance(&deps.querier, env.contract.address)?,
    })
}

// pub fn query_token_value(
//     deps: Deps,
//     env: Env,
//     identifier: AssetEntry,
//     amount: Option<Uint128>,
// ) -> ProxyResult<TokenValueResponse> {
//     let oracle = Oracle::new();
//     let ans_host = ANS_HOST.load(deps.storage)?;
//     let asset_info = identifier.resolve(&deps.querier, &ans_host)?;
//     let balance = asset_info.query_balance(&deps.querier, env.contract.address)?;
//     let asset = Asset {
//         info: asset_info,
//         amount: amount.unwrap_or(balance),
//     };
//     Ok(TokenValueResponse {
//         value: oracle.asset_value(deps, asset)?,
//     })
// }
