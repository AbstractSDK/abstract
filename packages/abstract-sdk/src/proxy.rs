//! # Proxy Helpers
use abstract_os::{
    ibc_client,
    objects::{proxy_asset::ProxyAsset, AssetEntry},
    proxy::{state::VAULT_ASSETS, AssetsResponse, ExecuteMsg, QueryMsg, TotalValueResponse},
};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, QuerierWrapper, QueryRequest, StdError, StdResult, Uint128,
    WasmMsg, WasmQuery,
};
use cw_storage_plus::Item;

use crate::{OsAction, ADMIN};
// Re-export os-id query as proxy is also core-contract.
pub use crate::manager::query_os_id;
/// Constructs the proxy dapp action message to execute CosmosMsgs on the Proxy.
pub fn os_module_action(msgs: Vec<CosmosMsg>, proxy_address: &Addr) -> StdResult<OsAction> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        msg: to_binary(&ExecuteMsg::ModuleAction { msgs })?,
        funds: vec![],
    }))
}

pub fn os_ibc_action(
    msgs: Vec<ibc_client::ExecuteMsg>,
    proxy_address: &Addr,
) -> StdResult<OsAction> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        msg: to_binary(&ExecuteMsg::IbcAction { msgs })?,
        funds: vec![],
    }))
}

/// Get the manager of the proxy contract
/// Admin always set to manager contract
pub fn query_os_manager_address(querier: &QuerierWrapper, proxy_address: &Addr) -> StdResult<Addr> {
    Item::new(ADMIN).query(querier, proxy_address.clone())
}

/// Query the total value denominated in the base asset
/// The provided address must implement the TotalValue Query
pub fn query_total_value(deps: Deps, proxy_address: &Addr) -> StdResult<Uint128> {
    let response: TotalValueResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: proxy_address.to_string(),
            msg: to_binary(&QueryMsg::TotalValue {})?,
        }))?;

    Ok(response.value)
}

/// RawQuery the proxy for a ProxyAsset
pub fn query_proxy_asset_raw(
    deps: Deps,
    proxy_address: &Addr,
    asset: &AssetEntry,
) -> StdResult<ProxyAsset> {
    let response = VAULT_ASSETS.query(&deps.querier, proxy_address.clone(), asset.clone())?;
    response.ok_or_else(|| {
        StdError::generic_err(format!(
            "Asset {} is not registered as an asset on your proxy contract.",
            asset
        ))
    })
}

/// List ProxyAssets raw
pub fn query_enabled_asset_names(
    deps: Deps,
    proxy_address: &Addr,
) -> StdResult<(Vec<AssetEntry>, AssetEntry)> {
    let mut asset_keys = vec![];
    let mut base_asset: Option<AssetEntry> = None;
    let mut resp: AssetsResponse = deps.querier.query_wasm_smart(
        proxy_address,
        &QueryMsg::Assets {
            page_token: None,
            page_size: None,
        },
    )?;
    while !resp.assets.is_empty() {
        let page_token = resp.assets.last().unwrap().0.clone();
        for (k, v) in resp.assets {
            maybe_set_base(&v, &mut base_asset);
            asset_keys.push(k);
        }
        resp = deps.querier.query_wasm_smart(
            proxy_address,
            &QueryMsg::Assets {
                page_token: Some(page_token.to_string()),
                page_size: None,
            },
        )?;
    }
    Ok((asset_keys, base_asset.unwrap()))
}

/// List ProxyAssets raw
pub fn query_enabled_assets(
    deps: Deps,
    proxy_address: &Addr,
) -> StdResult<Vec<(AssetEntry, ProxyAsset)>> {
    let mut assets = vec![];
    let mut resp: AssetsResponse = deps.querier.query_wasm_smart(
        proxy_address,
        &QueryMsg::Assets {
            page_token: None,
            page_size: None,
        },
    )?;
    while !resp.assets.is_empty() {
        let page_token = resp.assets.last().unwrap().0.clone();
        assets.append(resp.assets.as_mut());
        resp = deps.querier.query_wasm_smart(
            proxy_address,
            &QueryMsg::Assets {
                page_token: Some(page_token.to_string()),
                page_size: None,
            },
        )?;
    }
    Ok(assets)
}

#[inline(always)]
fn maybe_set_base(value: &ProxyAsset, base: &mut Option<AssetEntry>) {
    if value.value_reference.is_none() {
        *base = Some(value.asset.clone());
    }
}
