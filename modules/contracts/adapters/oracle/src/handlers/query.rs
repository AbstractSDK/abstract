use crate::{
    contract::{OracleAdapter, OracleResult},
    oracles::oracle_resolver::OracleAssetPrice,
};
use abstract_core::objects::{AssetEntry, DexAssetPairing, PoolAddress};
use abstract_oracle_standard::{
    msg::{OracleConfig, OracleQueryMsg, ProxyOrAddr, TokensValueResponse},
    state::{Config, Oracle},
};
use abstract_sdk::{features::AbstractNameService, Resolve};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

pub fn query_handler(
    deps: Deps,
    env: Env,
    adapter: &OracleAdapter,
    msg: OracleQueryMsg,
) -> OracleResult<Binary> {
    match msg {
        OracleQueryMsg::Config { proxy_address } => query_config(deps, proxy_address),
        OracleQueryMsg::TotalValue { proxy_address } => query_total_value(deps, proxy_address),
        OracleQueryMsg::TokensValue {
            proxy_or_address,
            identifiers,
        } => todo!(),
        OracleQueryMsg::HoldingAmounts {
            proxy_or_address,
            identifiers,
        } => todo!(),
        OracleQueryMsg::AssetPriceSources {
            proxy_address,
            identifier,
        } => todo!(),
        OracleQueryMsg::AssetIdentifiers {
            proxy_address,
            start_after,
            limit,
        } => todo!(),
        OracleQueryMsg::BaseAsset { proxy_address } => todo!(),
    }
}

pub fn query_config(deps: Deps, proxy_address: Option<String>) -> OracleResult<Binary> {
    let oracle = proxy_address
        .as_deref()
        .map(Oracle::new)
        .unwrap_or_default();
    let Config { external_age_max } = oracle.load_config(deps)?;
    to_json_binary(&OracleConfig { external_age_max }).map_err(Into::into)
}

pub fn query_total_value(deps: Deps, proxy_address: String) -> OracleResult<Binary> {
    let proxy_address = deps.api.addr_validate(&proxy_address)?;
    let mut oracle = Oracle::new(proxy_address.as_str());
    let value = oracle.account_value(deps)?;
    to_json_binary(&value).map_err(Into::into)
}

/// Returns the value of the amount of the specified assets
/// @param amount: The amount of the asset to compute the value of. If None, balance of the proxy account is used.
pub fn query_tokens_value(
    deps: Deps,
    adapter: &OracleAdapter,
    proxy_or_address: ProxyOrAddr,
    identifiers: Vec<AssetEntry>,
) -> OracleResult<TokensValueResponse> {
    let (address, oracle) = match &proxy_or_address {
        ProxyOrAddr::Proxy(proxy_addr_str) => {
            let proxy_addr = deps.api.addr_validate(proxy_addr_str)?;
            (proxy_addr, Oracle::new(proxy_addr_str))
        }
        ProxyOrAddr::Addr(addr_str) => {
            let addr = deps.api.addr_validate(addr_str)?;
            (addr, Oracle::default())
        }
    };
    let ans_host = adapter.ans_host(deps)?;
    for asset_entry in identifiers {
        let asset_info = asset_entry.resolve(&deps.querier, &ans_host)?;
        let balance = asset_info.query_balance(&deps.querier, &address)?;
        let value = oracle.asset_value(deps, Asset::new(asset_info, balance))?;
    }

    Ok(TokensValueResponse {
        tokens_value: todo!(),
        external_tokens_value: todo!(),
    })
}
