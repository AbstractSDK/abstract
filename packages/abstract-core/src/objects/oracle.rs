use std::collections::HashSet;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Order, StdError, StdResult, Uint128};
use cw_asset::{Asset, AssetInfo};
use cw_storage_plus::{Bound, Map};

use crate::AbstractResult;

use super::{
    ans_host::AnsHost,
    price_source::{AssetConversion, PriceSource, UncheckedPriceSource},
    AssetEntry,
};

pub type Complexity = u8;

pub const LIST_SIZE_LIMIT: u8 = 15;
const DEFAULT_PAGE_LIMIT: u8 = 5;

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
    /// Vec instead of HashMap because it's faster for small sets + AssetInfo does not implement `Hash`!
    asset_equivalent_cache: Vec<(AssetInfo, Vec<(AssetInfo, Uint128)>)>,
}

impl<'a> Oracle<'a> {
    pub const fn new() -> Self {
        Oracle {
            config: Map::new("oracle_config"),
            assets: Map::new("assets"),
            complexity: Map::new("complexity"),
            asset_equivalent_cache: Vec::new(),
        }
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
            return Err(crate::AbstractError::Std(StdError::generic_err(
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
            // Get dependencies for this price source
            let dependencies = price_source.dependencies(&asset);
            self.assert_dependencies_exists(deps.as_ref(), &dependencies)?;
            // get the complexity of the dependencies
            // depending on the type of price source, the complexity is calculated differently
            let complexity = self.asset_complexity(deps.as_ref(), &price_source, &dependencies)?;
            // Add asset to complexity level
            self.complexity.update(deps.storage, complexity, |v| {
                let mut v = v.unwrap_or_default();
                if v.contains(&asset) {
                    return Err(StdError::generic_err(format!(
                        "Asset {asset} already registered"
                    )));
                }
                v.push(asset.clone());
                Result::<_, StdError>::Ok(v)
            })?;
            self.assets.update(deps.storage, &asset, |v| {
                if v.is_some() {
                    return Err(StdError::generic_err(format!(
                        "asset {asset} already registered"
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
                return Err(StdError::generic_err(format!(
                    "Asset {asset} not registered on oracle"
                ))
                .into());
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

    /// Calculates the value of a single asset by recursive conversion to underlying asset(s).
    /// Does not make use of the cache to prevent querying the same price source multiple times.
    pub fn asset_value(&self, deps: Deps, asset: Asset) -> AbstractResult<Uint128> {
        // get the price source for the asset
        let (price_source, _) = self.assets.load(deps.storage, &asset.info)?;
        // get the conversions for this asset
        let conversion_rates = price_source.conversion_rates(deps, &asset.info)?;
        if conversion_rates.is_empty() {
            // no conversion rates means this is the base asset, return the amount
            return Ok(asset.amount);
        }
        // convert the asset into its underlying assets using the conversions
        let converted_assets = AssetConversion::convert(&conversion_rates, asset.amount);
        // recursively calculate the value of the underlying assets
        converted_assets
            .into_iter()
            .map(|a| self.asset_value(deps, a))
            .sum()
    }

    /// Calculates the total value of an account's assets by efficiently querying the configured price sources
    ///
    ///
    /// ## Resolve the total value of an account given a base asset.
    /// This process goes as follows
    /// 1. Get the assets for the highest, not visited, complexity.
    /// 2. For each asset query it's balance, get the conversion ratios associated with that asset and load its cached values.
    /// 3. Using the conversion ratio convert the balance and cached values and save the resulting values in the cache for that lower complexity asset.
    /// 4. Repeat until the base asset is reached. (complexity = 0)
    pub fn account_value(&mut self, deps: Deps, account: &Addr) -> AbstractResult<AccountValue> {
        // get the highest complexity
        let start_complexity = self.highest_complexity(deps)?;
        eprintln!("start complexity: {start_complexity}");
        self.complexity_value_calculation(deps, start_complexity, account)
    }

    /// Calculates the values of assets for a given complexity level
    fn complexity_value_calculation(
        &mut self,
        deps: Deps,
        complexity: u8,
        account: &Addr,
    ) -> AbstractResult<AccountValue> {
        let assets = self.complexity.load(deps.storage, complexity)?;
        for asset in assets {
            let (price_source, _) = self.assets.load(deps.storage, &asset)?;
            // get the balance for this asset
            let balance = asset.query_balance(&deps.querier, account)?;
            eprintln!("{asset}: {balance} ");
            // and the cached balances
            let mut cached_balances = self.cached_balance(&asset).unwrap_or_default();
            eprintln!("cached: {cached_balances:?}");
            // add the balance to the cached balances
            cached_balances.push((asset.clone(), balance));

            // get the conversion rates for this asset
            let conversion_rates = price_source.conversion_rates(deps, &asset)?;
            if conversion_rates.is_empty() {
                // no conversion rates means this is the base asset, construct the account value and return
                let total: u128 = cached_balances
                    .iter()
                    .map(|(_, amount)| amount.u128())
                    .sum::<u128>();

                return Ok(AccountValue {
                    total_value: Asset::new(asset, total),
                    breakdown: cached_balances,
                });
            }
            // convert the balance and cached values to this asset using the conversion rates
            self.update_cache(cached_balances, conversion_rates)?;
        }
        // call recursively for the next complexity level
        self.complexity_value_calculation(deps, complexity - 1, account)
    }

    /// Get the cached balance for an asset
    /// Removes from cache if present
    fn cached_balance(&mut self, asset: &AssetInfo) -> Option<Vec<(AssetInfo, Uint128)>> {
        let asset_pos = self
            .asset_equivalent_cache
            .iter()
            .position(|(asset_info, _)| asset_info == asset);
        asset_pos.map(|ix| self.asset_equivalent_cache.swap_remove(ix).1)
    }

    /// for each balance, convert it to the equivalent value in the target asset(s) of lower complexity
    /// update the cache of these target assets to include the re-valued balance of the source asset
    fn update_cache(
        &mut self,
        source_asset_balances: Vec<(AssetInfo, Uint128)>,
        conversions: Vec<AssetConversion>,
    ) -> AbstractResult<()> {
        eprintln!("updating cache with source asset balances: {source_asset_balances:?}");
        for (source_asset, balance) in source_asset_balances {
            // these balances are the equivalent to the source asset, just in a different denomination
            let target_assets_balances = AssetConversion::convert(&conversions, balance);
            // update the cache with these balances
            for Asset {
                info: target_asset,
                amount: balance,
            } in target_assets_balances
            {
                let cache = self
                    .asset_equivalent_cache
                    .iter_mut()
                    .find(|(a, _)| a == &target_asset);
                if let Some((_, cache)) = cache {
                    cache.push((source_asset.clone(), balance));
                } else {
                    self.asset_equivalent_cache
                        .push((target_asset, vec![(source_asset.clone(), balance)]));
                }
            }
        }
        eprintln!("cache updated: {:?}", self.asset_equivalent_cache);
        Ok(())
    }

    /// Checks that the oracle is configured correctly.
    pub fn validate(&self, deps: Deps) -> AbstractResult<()> {
        // no need to validate config as its assets are validated on add operations

        // fist check that a base asset is registered
        let base_asset = self.base_asset(deps)?;

        // Then start with lowest complexity assets and keep track of all the encountered assets.
        // If an asset has a dependency that is not in the list of encountered assets
        // then the oracle is not configured correctly.
        let mut encountered_assets: HashSet<String> = HashSet::from([base_asset.to_string()]);
        let max_complexity = self.highest_complexity(deps)?;
        // if only base asset, just return
        if max_complexity == 0 {
            return Ok(());
        }

        let mut complexity = 1;
        while complexity <= max_complexity {
            let assets = self.complexity.load(deps.storage, complexity)?;

            for asset in assets {
                let (price_source, _) = self.assets.load(deps.storage, &asset)?;
                let deps = price_source.dependencies(&asset);
                for dep in &deps {
                    if !encountered_assets.contains(&dep.to_string()) {
                        return Err(StdError::generic_err(format!(
                            "Asset {dep} is an oracle dependency but is not registered"
                        ))
                        .into());
                    }
                }
                if !encountered_assets.insert(asset.to_string()) {
                    return Err(StdError::generic_err(format!(
                        "Asset {asset} is registered twice"
                    ))
                    .into());
                };
            }
            complexity += 1;
        }
        Ok(())
    }

    /// Asserts that all dependencies of an asset are registered.
    fn assert_dependencies_exists(
        &self,
        deps: Deps,
        dependencies: &Vec<AssetInfo>,
    ) -> AbstractResult<()> {
        for dependency in dependencies {
            let asset_info = self.assets.has(deps.storage, dependency);
            if !asset_info {
                return Err(crate::AbstractError::Std(StdError::generic_err(format!(
                    "Asset {dependency} not registered on oracle"
                ))));
            }
        }
        Ok(())
    }

    // ### Queries ###

    /// Page over the oracle assets
    pub fn paged_asset_info(
        &self,
        deps: Deps,
        last_asset: Option<AssetInfo>,
        limit: Option<u8>,
    ) -> AbstractResult<Vec<(AssetInfo, (PriceSource, Complexity))>> {
        let limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(LIST_SIZE_LIMIT) as usize;
        let start_bound = last_asset.as_ref().map(Bound::exclusive);

        let res: Result<Vec<(AssetInfo, (PriceSource, Complexity))>, _> = self
            .assets
            .range(deps.storage, start_bound, None, Order::Ascending)
            .take(limit)
            .collect();

        res.map_err(Into::into)
    }

    /// Page over the oracle's asset configuration
    pub fn paged_asset_config(
        &self,
        deps: Deps,
        last_asset: Option<AssetEntry>,
        limit: Option<u8>,
    ) -> AbstractResult<Vec<(AssetEntry, UncheckedPriceSource)>> {
        let limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(LIST_SIZE_LIMIT) as usize;
        let start_bound = last_asset.as_ref().map(Bound::exclusive);

        let res: Result<Vec<(AssetEntry, UncheckedPriceSource)>, _> = self
            .config
            .range(deps.storage, start_bound, None, Order::Ascending)
            .take(limit)
            .collect();

        res.map_err(Into::into)
    }
    /// Get the highest complexity present in the oracle
    fn highest_complexity(&self, deps: Deps) -> AbstractResult<u8> {
        Ok(self
            .complexity
            .keys(deps.storage, None, None, Order::Descending)
            .take(1)
            .collect::<StdResult<Vec<u8>>>()?[0])
    }

    /// get the configuration of an asset
    pub fn asset_config(
        &self,
        deps: Deps,
        asset: &AssetEntry,
    ) -> AbstractResult<UncheckedPriceSource> {
        self.config.load(deps.storage, asset).map_err(Into::into)
    }

    pub fn base_asset(&self, deps: Deps) -> AbstractResult<AssetInfo> {
        let base_asset = self.complexity.load(deps.storage, 0);
        let Ok(base_asset) = base_asset else {
            return Err(StdError::generic_err("No base asset registered").into());
        };
        let base_asset_len = base_asset.len();
        if base_asset_len != 1 {
            return Err(StdError::generic_err(format!(
                "{base_asset_len} base assets registered, must be 1"
            ))
            .into());
        }
        Ok(base_asset[0].clone())
    }
}

#[cw_serde]
pub struct AccountValue {
    /// the total value of this account in the base denomination
    pub total_value: Asset,
    /// Vec of asset information and their value in the base asset denomination
    pub breakdown: Vec<(AssetInfo, Uint128)>,
}

// TODO: See if we can change this to multi-indexed maps when documentation improves.

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
    use cosmwasm_std::coin;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::Addr;

    use cosmwasm_std::Decimal;

    use speculoos::prelude::*;

    use crate::objects::DexAssetPairing;

    use super::*;
    type AResult = anyhow::Result<()>;

    pub fn get_ans() -> AnsHost {
        let addr = Addr::unchecked(TEST_ANS_HOST);

        AnsHost::new(addr)
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

    pub fn asset_as_half() -> (AssetEntry, UncheckedPriceSource) {
        let asset = AssetEntry::from(EUR);
        let price_source = UncheckedPriceSource::ValueAs {
            asset: AssetEntry::new(USD),
            multiplier: Decimal::percent(50),
        };
        (asset, price_source)
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
            .update_assets(deps.as_mut(), &ans, vec![asset_with_dep()], vec![])
            .unwrap_err();
        // add base asset
        oracle.update_assets(deps.as_mut(), &ans, vec![base_asset()], vec![])?;

        // try add second base asset, fails
        oracle
            .update_assets(deps.as_mut(), &ans, vec![base_asset()], vec![])
            .unwrap_err();
        // add asset with dependency
        oracle.update_assets(deps.as_mut(), &ans, vec![asset_with_dep()], vec![])?;

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

    #[test]
    fn query_base_value() -> AResult {
        let mut deps = mock_dependencies();
        let mock_ans = MockAnsHost::new().with_defaults();
        deps.querier = mock_ans.to_querier();
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, vec![coin(1000, USD)]);
        let ans = get_ans();
        let mut oracle = Oracle::new();

        // add base asset
        oracle.update_assets(deps.as_mut(), &ans, vec![base_asset()], vec![])?;

        let value = oracle.account_value(deps.as_ref(), &Addr::unchecked(MOCK_CONTRACT_ADDR))?;
        assert_that!(value.total_value.amount.u128()).is_equal_to(1000u128);

        let base_asset = oracle.base_asset(deps.as_ref())?;
        assert_that!(base_asset).is_equal_to(AssetInfo::native(USD));

        // get the one-asset value of the base asset
        let asset_value =
            oracle.asset_value(deps.as_ref(), Asset::new(AssetInfo::native(USD), 1000u128))?;
        assert_that!(asset_value.u128()).is_equal_to(1000u128);
        Ok(())
    }

    #[test]
    fn query_equivalent_asset_value() -> AResult {
        let mut deps = mock_dependencies();
        let mock_ans = MockAnsHost::new().with_defaults();
        deps.querier = mock_ans.to_querier();
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, vec![coin(1000, EUR)]);
        let ans = get_ans();
        let mut oracle = Oracle::new();
        // fails because base asset is not set.
        let res = oracle.update_assets(deps.as_mut(), &ans, vec![asset_as_half()], vec![]);
        // match when adding better errors
        assert_that!(res).is_err();
        // fails, need to add base asset first, TODO: try removing this requirement when more tests are added.
        oracle
            .update_assets(
                deps.as_mut(),
                &ans,
                vec![asset_as_half(), base_asset()],
                vec![],
            )
            .unwrap_err();

        // now in correct order
        oracle.update_assets(
            deps.as_mut(),
            &ans,
            vec![base_asset(), asset_as_half()],
            vec![],
        )?;

        let value = oracle.account_value(deps.as_ref(), &Addr::unchecked(MOCK_CONTRACT_ADDR))?;
        assert_that!(value.total_value.amount.u128()).is_equal_to(500u128);

        // give the account some base asset
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, vec![coin(1000, USD), coin(1000, EUR)]);

        // assert that the value increases with 1000
        let value = oracle.account_value(deps.as_ref(), &Addr::unchecked(MOCK_CONTRACT_ADDR))?;
        assert_that!(value.total_value.amount.u128()).is_equal_to(1500u128);

        // get the one-asset value of the base asset
        let asset_value =
            oracle.asset_value(deps.as_ref(), Asset::new(AssetInfo::native(USD), 1000u128))?;
        assert_that!(asset_value.u128()).is_equal_to(1000u128);

        // now for EUR
        let asset_value =
            oracle.asset_value(deps.as_ref(), Asset::new(AssetInfo::native(EUR), 1000u128))?;
        assert_that!(asset_value.u128()).is_equal_to(500u128);
        Ok(())
    }

    // test for pair

    // test for LP tokens

    // test for max complexity
}
