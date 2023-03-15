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

use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    str::FromStr,
};

use cosmwasm_std::{Deps, DepsMut, Order, StdError, StdResult, Uint128};
use cw_asset::AssetInfo;
use cw_storage_plus::{Index, IndexList, IndexedMap, Map, MultiIndex, PrimaryKey, UniqueIndex};
use serde::{Deserialize, Serialize};

use crate::AbstractResult;

use super::{
    ans_host::AnsHost,
    price_source::{PriceSource, UncheckedPriceSource},
    AssetEntry,
};

pub type Complexity = u8;

const LIST_SIZE_LIMIT: usize = 15;

type HashableAssetInfo = String;

/// Struct for calculating asset prices/values for a smart contract.
pub struct Oracle<'a> {
    /// map of human-readable asset names to their human-readable price source
    pub config: Map<'static, &'a AssetEntry, UncheckedPriceSource>,
    /// Assets map to get the complexity and value calculation of an asset.
    assets: Map<'static, &'a AssetInfo, (PriceSource, Complexity)>,
    /// Complexity rating used for efficient total value calculation
    /// Vec > HashSet because it's faster for small sets
    complexity: Map<'static, Complexity, Vec<AssetInfo>>,
    /// Cache of asset values for efficient total value calculation
    /// the amount set for an asset will be added to its balance.
    asset_equivalent_cache: Option<HashMap<HashableAssetInfo, Vec<(Uint128, HashableAssetInfo)>>>,
}

impl<'a> Oracle<'a> {
    const fn new() -> Self {
        Oracle {
            config: Map::new("oracle_config"),
            assets: Map::new("assets"),
            complexity: Map::new("complexity"),
            asset_equivalent_cache: None,
        }
    }

    /// Instantiate the oracle cache
    fn with_cache(mut self) -> Self {
        self.asset_equivalent_cache = Some(HashMap::new());
        self
    }

    pub fn update_assets(
        &self,
        mut deps: DepsMut,
        ans: &AnsHost,
        to_add: Vec<(AssetEntry, UncheckedPriceSource)>,
        to_remove: Vec<AssetEntry>,
    ) -> AbstractResult<()> {
        let current_vault_size = self
            .config
            .keys(deps.storage, None, None, Order::Ascending)
            .count();
        let delta: i128 = to_add.len() as i128 - to_remove.len() as i128;
        if current_vault_size as i128 + delta > LIST_SIZE_LIMIT as i128 {
            return Err(crate::AbstractOsError::Std(StdError::generic_err(
                "Oracle list size limit exceeded",
            )));
        }
        // remove assets from oracle
        self.remove_assets(deps.branch(), ans, to_remove)?;
        // add assets to oracle
        self.add_assets(deps.branch(), ans, to_add)?;
        // validate the oracle configuration
        // Each asset must have a valid price source
        // and there can only be one base asset.
        self.validate(deps.as_ref())
    }

    /// Adds assets to the oracle
    fn add_assets(
        &self,
        deps: DepsMut,
        ans: &AnsHost,
        assets: Vec<(AssetEntry, UncheckedPriceSource)>,
    ) -> AbstractResult<()> {
        // optimistically update config
        // configuration check happens after all updates have been done.
        for (key, data) in assets.iter() {
            self.config.save(deps.storage, key, data)?;
        }

        let (assets, price_sources): (Vec<AssetEntry>, Vec<_>) = assets.into_iter().unzip();
        let resolved_assets = ans.query_assets(&deps.querier, &assets)?;

        let checked_price_sources = price_sources
            .into_iter()
            .enumerate()
            .map(|(ix, price_source)| price_source.check(deps.as_ref(), ans, &assets[ix]))
            .collect::<Result<Vec<PriceSource>, _>>()?;

        let assets_and_sources = resolved_assets
            .into_iter()
            .zip(checked_price_sources)
            .collect::<Vec<_>>();

        // Now that we validated the input, assign a complexity to them and add them to the oracle

        // Register asset
        // Registration is expected to be done in increasing complexity
        // So this will fail if a dependent asset is not registered first.
        for (asset, price_source) in assets_and_sources {
            let dependencies;
                    // Get dependencies for this price source
                    dependencies = price_source.dependency(&asset);
                    self.assert_dependency_exists(deps.as_ref(), &dependencies)?;
                    // get the complexity of the dependencies
                    // depending on the type of price source, the complexity is calculated differently
                    let complexity =
                        self.asset_complexity(deps.as_ref(), &price_source, &dependencies)?;
                    // Add asset to complexity level
                    self.complexity.update(deps.storage, complexity, |v| {
                        let mut v = v.unwrap_or_default();
                        if v.contains(&asset) {
                            return Err(StdError::generic_err(format!(
                                "Asset {} already registered",
                                asset
                            )));
                        }
                        v.push(asset.clone());
                        Result::<_, StdError>::Ok(v)
                    })?;
                    self.assets.update(deps.storage, &asset, |v| {
                        if v.is_some() {
                            return Err(StdError::generic_err(format!(
                                "asset {} already registered",
                                asset
                            )));
                        }
                        Ok((price_source, complexity))
                    })?;
        }

        Ok(())
    }

    /// Removes assets from the oracle
    fn remove_assets(
        &self,
        deps: DepsMut,
        ans: &AnsHost,
        assets: Vec<AssetEntry>,
    ) -> AbstractResult<()> {
        for asset in assets {
            // assert asset was in config
            if !self.config.has(deps.storage, &asset) {
                return Err(
                    StdError::generic_err(format!("Asset {} not registered", asset)).into(),
                );
            }
            // remove from config
            self.config.remove(deps.storage, &asset);
            // get its asset information
            let asset = ans.query_asset(&deps.querier, &asset)?;
            // get its complexity
            let (_, complexity) = self.assets.load(deps.storage, &asset)?;
            // remove from assets
            self.assets.remove(deps.storage, &asset);
            // remove from complexity level
            self.complexity.update(deps.storage, complexity, |v| {
                let mut v = v.unwrap_or_default();
                v.retain(|a| a != &asset);
                Result::<_, StdError>::Ok(v)
            })?;
        }
        Ok(())
    }

    /// Returns the complexity of an asset
    // Complexity logic:
    // base: 0
    // Pair: paired asset + 1
    // LP: highest in pool + 1
    // ValueAs: equal asset + 1
    fn asset_complexity(
        &self,
        deps: Deps,
        price_source: &PriceSource,
        dependencies: &[AssetInfo],
    ) -> AbstractResult<Complexity> {
        match price_source {
            PriceSource::None => Ok(0),
            PriceSource::Pool { .. } => {
                let compl = self.assets.load(deps.storage, &dependencies[0])?.1;
                Ok(compl + 1)
            }
            PriceSource::LiquidityToken { .. } => {
                let mut max = 0;
                for dependency in dependencies {
                    let (_, complexity) = self.assets.load(deps.storage, dependency)?;
                    if complexity > max {
                        max = complexity;
                    }
                }
                Ok(max + 1)
            }
            PriceSource::ValueAs { asset, .. } => {
                let (_, complexity) = self.assets.load(deps.storage, asset)?;
                Ok(complexity + 1)
            }
        }
    }

    /// Calculates the value of a single asset by recursive conversion to underlying assets.
    pub fn asset_value(&self, asset: AssetEntry) {}

    /// Calculates the total value of an account's assets by
    pub fn account_value(&mut self, account: String) {}

    pub fn assets_info(&self) {}

    /// Checks that the oracle is configured correctly.
    pub fn validate(&self, deps: Deps) -> AbstractResult<()> {
        // no need to validate config as its assets are validated on add operations

        // fist check that a base asset is registered
        let base_asset = self.complexity.load(deps.storage, 0)?;
        let base_asset_len = base_asset.len();
        if base_asset_len != 0 {
            return Err(StdError::generic_err(
                "{base_asset_len} base assets registered, must be 1",
            )
            .into());
        }

        // Then start with lowest complexity assets and keep track of all the encountered assets.
        // If an asset has a dependency that is not in the list of encountered assets
        // then the oracle is not configured correctly.
        let mut encountered_assets: HashSet<String> =
            base_asset.iter().map(ToString::to_string).collect();
        let max_complexity = self.hightest_complexity(deps)?;
        // if only base asset, just return
        if max_complexity == 0 {
            return Ok(());
        }

        let mut complexity = 1;
        while complexity <= max_complexity {
            let assets = self.complexity.load(deps.storage, complexity)?;

            for asset in assets {
                let (price_source, _) = self.assets.load(deps.storage, &asset)?;
                let deps = price_source.dependency(&asset);
                for dep in &deps {
                    if !encountered_assets.contains(&dep.to_string()) {
                        return Err(StdError::generic_err(format!(
                            "Asset {} is a dependency but is not registered",
                            dep
                        ))
                        .into());
                    }
                }
                if !encountered_assets.insert(asset.to_string()) {
                    return Err(StdError::generic_err(format!(
                        "Asset {} is registered twice",
                        asset
                    ))
                    .into());
                };
            }
            complexity += 1;
        }
        Ok(())
    }

    /// Asserts that all dependencies of an asset are registered.
    fn assert_dependency_exists(
        &self,
        deps: Deps,
        dependencies: &Vec<AssetInfo>,
    ) -> AbstractResult<()> {
        for dependency in dependencies {
            let asset_info = self.assets.has(deps.storage, dependency);
            if !asset_info {
                return Err(crate::AbstractOsError::Std(StdError::generic_err(format!(
                    "Asset {dependency} not registered"
                ))));
            }
        }
        Ok(())
    }

    /// Get the highest complexity present in the oracle
    fn hightest_complexity(&self, deps: Deps) -> AbstractResult<u8> {
        Ok(self
            .complexity
            .keys(deps.storage, None, None, Order::Descending)
            .take(1)
            .collect::<StdResult<Vec<u8>>>()?[0])
    }
}

// See if we can change this to multi-indexed maps when documentation improves.

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// struct OracleAsset {
//     asset: AssetInfo,
//     price_source: PriceSource,
//     complexity: Complexity,
// }

// struct Foo<'a> {
//     map: IndexedMap<'a, &'a str, OracleAsset, OracleIndexes<'a> >
// }

// impl<'a> Foo<'a> {
//     fn new() -> Self {
//         let indexes = OracleIndexes {
//             complexity: MultiIndex::<'a>::new(
//                 |_pk ,d: &OracleAsset| d.complexity,
//                 "tokens",
//                 "tokens__owner",
//             ),
//             asset: UniqueIndex::<'_,AssetInfo,_,()>::new(|d: &OracleAsset| d.asset, "asset"),
//         };
//         IndexedMap::new("or_assets", indexes)
//         Self {  } }
// }

// struct OracleIndexes<'a> {
//     pub asset: UniqueIndex<'a, &'a AssetInfo, OracleAsset, String>,
//     pub complexity: MultiIndex<'a, u8, OracleAsset, String>,
// }

// impl<'a> IndexList<OracleAsset> for OracleIndexes<'a> {
//     fn get_indexes(&'_ self) ->Box<dyn Iterator<Item = &'_ dyn Index<OracleAsset>> + '_> {
//         let v: Vec<&dyn Index<_>> = vec![&self.asset, &self.complexity];
//         Box::new(v.into_iter())
//     }
// }
// pub fn oracle_asset_complexity<T>(_pk: &[u8], d: &OracleAsset) -> u8 {
//     d.complexity
// }

#[cfg(test)]
mod tests {
    use abstract_testing::prelude::EUR;
    use abstract_testing::prelude::TEST_ANS_HOST;
    use abstract_testing::prelude::TEST_DEX;
    use abstract_testing::prelude::USD;
    use abstract_testing::MockAnsHost;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::Addr;
    use speculoos::prelude::*;

    use crate::objects::DexAssetPairing;

    use super::*;
    type AResult = anyhow::Result<()>;

    pub fn get_ans() -> AnsHost {
        let addr = Addr::unchecked(TEST_ANS_HOST);
        let ans = AnsHost::new(addr);
        ans
    }

    pub fn base_asset() -> (AssetEntry, UncheckedPriceSource) {
        (AssetEntry::from(USD), UncheckedPriceSource::None)
    }

    pub fn asset_with_dep() -> (AssetEntry, UncheckedPriceSource) {
        let asset = AssetEntry::from(EUR);
        let price_source = UncheckedPriceSource::Pair(DexAssetPairing::new(
            AssetEntry::new(EUR),
            AssetEntry::new(USD),
            TEST_DEX,
        ));
        (asset, price_source)
    }

    pub fn assets() -> Vec<(AssetEntry, UncheckedPriceSource)> {
        vec![base_asset(), asset_with_dep()]
    }

    pub fn entries() -> Vec<AssetEntry> {
        vec![AssetEntry::from(USD), AssetEntry::from(EUR)]
    }

    #[test]
    fn add_base_asset() -> AResult {
        let mut deps = mock_dependencies();
        let mock_ans = MockAnsHost::new().with_defaults();
        deps.querier = mock_ans.to_querier();
        let ans = get_ans();

        let oracle = Oracle::new();
        // first asset can not have dependency
        oracle
            .add_assets(deps.as_mut(), &ans, vec![asset_with_dep()])
            .unwrap_err();
        // add base asset
        oracle.add_assets(deps.as_mut(), &ans, vec![base_asset()])?;

        // try add second base asset, fails
        oracle
            .add_assets(deps.as_mut(), &ans, vec![base_asset()])
            .unwrap_err();
        // add asset with dependency
        oracle.add_assets(deps.as_mut(), &ans, vec![asset_with_dep()])?;

        // ensure these assets were added
        // Ensure that all assets have been added to the oracle
        let assets = oracle
            .config
            .range(&deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        assert_that!(assets).has_length(2);
        assert_that!(assets[0].0.as_str()).is_equal_to(EUR);
        assert_that!(assets[0].1).is_equal_to(UncheckedPriceSource::Pair(DexAssetPairing::new(
            AssetEntry::new(EUR),
            AssetEntry::new(USD),
            TEST_DEX,
        )));
        assert_that!(assets[1].0.as_str()).is_equal_to(USD);
        assert_that!(assets[1].1).is_equal_to(UncheckedPriceSource::None);

        // Ensure that all assets have been added to the complexity index
        let complexity = oracle
            .complexity
            .range(&deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        // 2 assets, 1 base asset, 1 asset with dependency
        assert_that!(complexity).has_length(2);

        assert_that!(complexity[0].1).has_length(1);
        assert_that!(complexity[1].1).has_length(1);
        

        Ok(())
    }
}
