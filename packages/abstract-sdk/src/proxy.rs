//! # Proxy Helpers
use abstract_os::{
    objects::{proxy_asset::ProxyAsset, AssetEntry},
    proxy::{state::VAULT_ASSETS, ExecuteMsg, QueryMsg, QueryTotalValueResponse},
};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, Empty, QueryRequest, StdError, StdResult, Uint128, WasmMsg,
    WasmQuery,
};

// Re-export os-id query as proxy is also core-contract.
pub use crate::manager::query_os_id;
/// Constructs the proxy dapp action message to execute CosmosMsgs on the Proxy.
pub fn send_to_proxy(msgs: Vec<CosmosMsg>, proxy_address: &Addr) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        msg: to_binary(&ExecuteMsg::ModuleAction { msgs })?,
        funds: vec![],
    }))
}

/// Query the total value denominated in the base asset
/// The provided address must implement the TotalValue Query
pub fn query_total_value(deps: Deps, proxy_address: &Addr) -> StdResult<Uint128> {
    let response: QueryTotalValueResponse =
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
