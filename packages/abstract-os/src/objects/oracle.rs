// There are two functions the oracle must perform:

// ## Resolve the total value of an account given a base asset.
// This process goes as follows
// 1. Get the highest complexity asset and check the cache for a balance.
// 2. Get the price associated with that asset and convert it into its lower complexity equivalent.
// 3. Save the resulting value in the cache for that lower complexity asset.
// 4. Repeat until the base asset is reached.

// ## Resolve the value of a single asset.
// 1. Get the assets's price source
// 2. Get the price of the asset from the price source
// 3. Get the price source of the asset's equivalent asset
// 4. Repeat until the base asset is reached.

use std::collections::HashMap;

use cosmwasm_std::Uint128;
use cw_storage_plus::{Deque, Map};

use crate::proxy::state::VAULT_ASSETS;

use super::{proxy_asset::ProxyAsset, AssetEntry};

pub struct Oracle<'a> {
    asset_equivalent_cache: HashMap<AssetEntry, Uint128>,
    assets: Map<'static, &'a AssetEntry, ProxyAsset>,
    complexity: Map<'static, u16, Vec<AssetEntry>>,
}

impl<'a> Oracle<'a> {
    pub fn new() -> Self {
        Oracle {
            asset_equivalent_cache: HashMap::new(),
            assets: VAULT_ASSETS,
            complexity: Map::new("complexity"),
        }
    }

    pub fn asset_value(&self, asset: AssetEntry) {}

    pub fn account_value(&self, account: String) {}
}
