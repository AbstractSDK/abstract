//! # Vault
//! The Vault object provides function for querying balances and asset values for the OS.

use crate::{
    cw_helpers::cosmwasm_std::wasm_smart_query,
    features::{AbstractNameService, Identification},
    AbstractSdkResult,
};
use abstract_os::{objects::AssetEntry, proxy::QueryMsg};
use cosmwasm_std::{Deps, Uint128};

use os::{
    objects::oracle::AccountValue,
    proxy::{BaseAssetResponse, TokenValueResponse},
};

/// Retrieve asset-registration information from the OS.
/// Query asset values and balances.
pub trait VaultInterface: AbstractNameService + Identification {
    fn vault<'a>(&'a self, deps: Deps<'a>) -> Vault<Self> {
        Vault { base: self, deps }
    }
}

impl<T> VaultInterface for T where T: AbstractNameService + Identification {}

#[derive(Clone)]
pub struct Vault<'a, T: VaultInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: VaultInterface> Vault<'a, T> {
    /// Query the total value denominated in the base asset
    /// The provided address must implement the TotalValue Query
    pub fn query_total_value(&self) -> AbstractSdkResult<AccountValue> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;
        let response: AccountValue = querier.query(&wasm_smart_query(
            proxy_address.to_string(),
            &QueryMsg::TotalValue {},
        )?)?;

        Ok(response)
    }

    /// Query the asset value denominated in the base asset
    pub fn asset_value(&self, asset_entry: AssetEntry) -> AbstractSdkResult<Uint128> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;
        let response: TokenValueResponse = querier.query(&wasm_smart_query(
            proxy_address.to_string(),
            &QueryMsg::TokenValue {
                identifier: asset_entry,
            },
        )?)?;

        Ok(response.value)
    }

    /// Return the proxy's base asset
    pub fn base_asset(&self) -> AbstractSdkResult<BaseAssetResponse> {
        let querier = self.deps.querier;
        let proxy_address = self.base.proxy_address(self.deps)?;
        let response: BaseAssetResponse = querier.query(&wasm_smart_query(
            proxy_address.to_string(),
            &QueryMsg::BaseAsset {},
        )?)?;

        Ok(response)
    }

    // List ProxyAssets smart
    // pub fn enabled_assets_list(&self) -> AbstractSdkResult<(Vec<AssetInfo>, AssetInfo)> {
    //     let querier = self.deps.querier;
    //     let proxy_address = self.base.proxy_address(self.deps)?;

    //     let mut asset_keys = vec![];
    //     let mut base_asset: Option<AssetInfo> = None;
    //     let mut resp: AssetsResponse = querier.query_wasm_smart(
    //         &proxy_address,
    //         &QueryMsg::Assets {
    //             start_after: None,
    //             limit: None,
    //         },
    //     )?;
    //     while !resp.assets.is_empty() {
    //         let start_after = resp.assets.last().unwrap().0.clone();
    //         for (k, v) in resp.assets {
    //             match v.price_source {
    //                 PriceSource::None => {
    //                 base_asset = Some(v.asset.clone());
    //             },
    //             _ => {}
    //         }
    //             asset_keys.push(k);
    //         }
    //         resp = querier.query_wasm_smart(
    //             &proxy_address,
    //             &QueryMsg::Assets {
    //                 start_after: Some(start_after.to_string()),
    //                 limit: None,
    //             },
    //         )?;
    //     }
    //     Ok((asset_keys, base_asset.unwrap()))
    // }

    // /// List ProxyAssets raw
    // pub fn proxy_assets_list(&self) -> AbstractSdkResult<Vec<(AssetEntry, ProxyAsset)>> {
    //     let querier = self.deps.querier;
    //     let proxy_address = self.base.proxy_address(self.deps)?;

    //     let mut assets = vec![];
    //     let mut resp: AssetsResponse = querier.query_wasm_smart(
    //         &proxy_address,
    //         &QueryMsg::Assets {
    //             start_after: None,
    //             limit: None,
    //         },
    //     )?;
    //     while !resp.assets.is_empty() {
    //         let start_after = resp.assets.last().unwrap().0.clone();
    //         assets.append(resp.assets.as_mut());
    //         resp = querier.query_wasm_smart(
    //             &proxy_address,
    //             &QueryMsg::Assets {
    //                 start_after: Some(start_after.to_string()),
    //                 limit: None,
    //             },
    //         )?;
    //     }
    //     Ok(assets)
    // }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::mock_module::*;

    mod query_total_value {
        // use super::*;
    }
}
