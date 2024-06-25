use std::collections::HashSet;

use abstract_sdk::feature_objects::AnsHost;
use abstract_std::{
    objects::{
        price_source::{AssetConversion, ExternalPriceSource, PriceSource, UncheckedPriceSource},
        AssetEntry,
    },
    AbstractError, AbstractResult,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Order, StdError, Uint128};
use cw_asset::{Asset, AssetInfo};
use cw_storage_plus::{Bound, Map};

use crate::msg::{AccountValue, Complexity, ProviderName};

#[cw_serde]
pub struct Config {
    pub external_age_max: u64,
}

// TODO: do we save it here or in ans?
pub const ADDRESSES_OF_PROVIDERS: Map<&ProviderName, Addr> = Map::new("providers");

pub const LIST_SIZE_LIMIT: u8 = 15;
const DEFAULT_PAGE_LIMIT: u8 = 5;

#[cw_serde]
pub struct OraclePriceSource {
    pub provider: String,
    pub price_source_key: String,
}

impl ExternalPriceSource for OraclePriceSource {
    fn check(&self, deps: Deps, ans_host: &AnsHost, entry: &AssetEntry) -> AbstractResult<()> {
        todo!()
    }
}

/// Struct for calculating asset prices/values for a smart contract.
pub struct Oracle<'a> {
    config: Map<'static, &'a str, Config>,
    /// map of human-readable asset names to their human-readable price source
    pub sources: Map<'static, (&'a str, &'a AssetEntry), UncheckedPriceSource<OraclePriceSource>>,
    /// Assets map to get the complexity and value calculation of an asset.
    assets: Map<'static, (&'a str, &'a AssetInfo), (PriceSource<OraclePriceSource>, Complexity)>,
    /// Complexity rating used for efficient total value calculation
    /// Vec > HashSet because it's faster for small sets
    complexity: Map<'static, (&'a str, Complexity), Vec<AssetInfo>>,
    /// Cache of asset values for efficient total value calculation
    /// the amount set for an asset will be added to its balance.
    /// Vec instead of HashMap because it's faster for small sets + AssetInfo does not implement `Hash`!
    asset_equivalent_cache: Vec<(AssetInfo, Vec<(AssetInfo, Uint128)>)>,
    user: &'a str,
}

impl<'a> Default for Oracle<'a> {
    fn default() -> Self {
        // Empty user - admin
        Self::new("")
    }
}

impl<'a> Oracle<'a> {
    /// Get Oracle object
    pub const fn new(user: &'a str) -> Self {
        Oracle {
            config: Map::new("config"),
            sources: Map::new("sources"),
            assets: Map::new("assets"),
            complexity: Map::new("complexity{postfix}"),
            asset_equivalent_cache: Vec::new(),
            user,
        }
    }

    /// Update oracle config for the user
    pub fn update_config(&self, deps: DepsMut, external_age_max: u64) -> AbstractResult<()> {
        self.config
            .save(deps.storage, self.user, &Config { external_age_max })
            .map_err(Into::into)
    }

    /// Load config
    /// Uses user defined config if present, or default if not
    pub fn load_config(&self, deps: Deps) -> AbstractResult<Config> {
        // Try to load user config
        if let Some(config) = self.config.may_load(deps.storage, self.user)? {
            Ok(config)
        } else {
            // Otherwise use default config
            self.config
                .load(deps.storage, Default::default())
                .map_err(Into::into)
        }
    }

    /// Updates the assets in the Oracle.
    /// First adds the provided assets to the oracle, then removes the provided assets from the oracle.
    pub fn update_assets(
        &self,
        mut deps: DepsMut,
        ans: &AnsHost,
        to_add: Vec<(AssetEntry, UncheckedPriceSource<OraclePriceSource>)>,
        to_remove: Vec<AssetEntry>,
    ) -> AbstractResult<()> {
        // If it's an user oracle - check if it 's in size limit
        if self.user != "" {
            let current_vault_size = self
                .sources
                .keys(deps.storage, None, None, Order::Ascending)
                .count();
            let new_vault_size = current_vault_size + to_add.len() - to_remove.len();
            if new_vault_size > LIST_SIZE_LIMIT as usize {
                return Err(AbstractError::Std(StdError::generic_err(
                    "Oracle list size limit exceeded",
                )));
            }
        }

        let mut all: Vec<&AssetEntry> = to_add.iter().map(|(a, _)| a).chain(&to_remove).collect();
        all.dedup();
        if all.len() != to_add.len() + to_remove.len() {
            return Err(AbstractError::Std(StdError::generic_err(
                "Duplicate assets in update",
            )));
        }

        // add assets to oracle
        self.add_assets(deps.branch(), ans, to_add)?;
        // remove assets from oracle
        self.remove_assets(deps.branch(), ans, to_remove)?;
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
        assets: Vec<(AssetEntry, UncheckedPriceSource<OraclePriceSource>)>,
    ) -> AbstractResult<()> {
        // optimistically update config
        // configuration check happens after all updates have been done.
        for (key, data) in assets.iter() {
            self.sources.save(deps.storage, (self.user, key), data)?;
        }

        let (assets, price_sources): (Vec<AssetEntry>, Vec<_>) = assets.into_iter().unzip();
        let resolved_assets = ans.query_assets(&deps.querier, &assets)?;

        let checked_price_sources = price_sources
            .into_iter()
            .enumerate()
            .map(|(ix, price_source)| price_source.check(deps.as_ref(), ans, &assets[ix]))
            .collect::<Result<Vec<PriceSource<OraclePriceSource>>, _>>()?;

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
            self.complexity
                .update(deps.storage, (self.user, complexity), |v| {
                    let mut v = v.unwrap_or_default();
                    if v.contains(&asset) {
                        return Err(StdError::generic_err(format!(
                            "Asset {asset} already registered"
                        )));
                    }
                    v.push(asset.clone());
                    Result::<_, StdError>::Ok(v)
                })?;
            self.assets.update(deps.storage, (self.user, &asset), |v| {
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
            if !self.sources.has(deps.storage, (self.user, &asset)) {
                return Err(StdError::generic_err(format!(
                    "Asset {asset} not registered on oracle"
                ))
                .into());
            }
            // remove from config
            self.sources.remove(deps.storage, (self.user, &asset));
            // get its asset information
            let asset = ans.query_asset(&deps.querier, &asset)?;
            // get its complexity
            let (_, complexity) = self.assets.load(deps.storage, (self.user, &asset))?;
            // remove from assets
            self.assets.remove(deps.storage, (&self.user, &asset));
            // remove from complexity level
            self.complexity
                .update(deps.storage, (&self.user, complexity), |v| {
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
    // External: 1, as it requires exactly 1 query to figure out USD value of it
    fn asset_complexity(
        &self,
        deps: Deps,
        price_source: &PriceSource<OraclePriceSource>,
        dependencies: &[AssetInfo],
    ) -> AbstractResult<Complexity> {
        match price_source {
            PriceSource::Base => Ok(0),
            PriceSource::Pool { .. } => {
                let compl = self
                    .assets
                    .load(deps.storage, (self.user, &dependencies[0]))?
                    .1;
                Ok(compl + 1)
            }
            PriceSource::LiquidityToken { .. } => {
                let mut max = 0;
                for dependency in dependencies {
                    let (_, complexity) =
                        self.assets.load(deps.storage, (self.user, dependency))?;
                    if complexity > max {
                        max = complexity;
                    }
                }
                Ok(max + 1)
            }
            PriceSource::ValueAs { asset, .. } => {
                let (_, complexity) = self.assets.load(deps.storage, (self.user, asset))?;
                Ok(complexity + 1)
            }
            PriceSource::External(_) => Ok(1),
        }
    }

    /// Calculates the value of a single asset by recursive conversion to underlying asset(s).
    /// Does not make use of the cache to prevent querying the same price source multiple times.
    pub fn asset_value(&self, deps: Deps, asset: Asset) -> AbstractResult<AssetValue> {
        // get the price source for the asset
        let (price_source, _) = self.assets.load(deps.storage, (self.user, &asset.info))?;
        match price_source {
            PriceSource::Base => {
                return Ok(AssetValue {
                    base: asset.amount,
                    external: Uint128::zero(),
                })
            }
            PriceSource::External(external) => {
                return Ok(AssetValue {
                    base: Uint128::zero(),
                    external: self.external_asset_value(external)?,
                })
            }
            _ => (),
        }
        // get the conversions for this asset
        let conversion_rates = price_source.conversion_rates(deps, &asset.info)?;
        // convert the asset into its underlying assets using the conversions
        let converted_assets = AssetConversion::convert(&conversion_rates, asset.amount);
        // recursively calculate the value of the underlying assets
        let sum: AbstractResult<AssetValue> = converted_assets
            .into_iter()
            .map(|a| self.asset_value(deps, a))
            .fold(Ok(AssetValue::default()), |acc, e| Ok(acc? + e?));
        todo!();
    }

    fn external_asset_value(
        &self,
        external_price_source: OraclePriceSource,
    ) -> AbstractResult<Uint128> {
        todo!()
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
    pub fn account_value(&mut self, deps: Deps) -> AbstractResult<TotalValue> {
        // fist check that a base asset is registered
        let _ = self.base_asset(deps)?;

        // get the highest complexity
        let start_complexity = self.highest_complexity(deps)?;
        self.complexity_value_calculation(deps, start_complexity, self.user)
    }

    /// Calculates the values of assets for a given complexity level
    fn complexity_value_calculation(
        &mut self,
        deps: Deps,
        complexity: u8,
        account: &str,
    ) -> AbstractResult<TokensValueResponse> {
        let assets = self
            .complexity
            .load(deps.storage, (self.user, complexity))?;
        for asset in assets {
            let (price_source, _) = self.assets.load(deps.storage, (self.user, &asset))?;
            // get the balance for this asset
            let balance = asset.query_balance(&deps.querier, account)?;
            // and the cached balances
            let mut cached_balances = self.cached_balance(&asset).unwrap_or_default();
            // add the balance to the cached balances
            cached_balances.push((asset.clone(), balance));

            if let PriceSource::Base = price_source {
                // no conversion rates means this is the base asset, construct the account value and return
                let total: Uint128 = cached_balances.iter().map(|(_, amount)| amount).sum();

                return Ok(TokensValueResponse {
                    tokens_value: TotalValue {
                        total_value: total,
                        breakdown: cached_balances,
                    },
                    // TODO:
                    external_tokens_value: TotalValue {
                        total_value: Uint128::zero(),
                        breakdown: vec![],
                    },
                });
            }
            // get the conversion rates for this asset
            let conversion_rates = price_source.conversion_rates(deps, &asset)?;
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
            let assets = self
                .complexity
                .load(deps.storage, (self.user, complexity))?;

            for asset in assets {
                let (price_source, _) = self.assets.load(deps.storage, (self.user, &asset))?;
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
            let asset_info = self.assets.has(deps.storage, (self.user, dependency));
            if !asset_info {
                return Err(AbstractError::Std(StdError::generic_err(format!(
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
    ) -> AbstractResult<Vec<(AssetInfo, (PriceSource<OraclePriceSource>, Complexity))>> {
        let limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(LIST_SIZE_LIMIT) as usize;
        let start_bound = last_asset.as_ref().map(Bound::exclusive);

        let res: Result<Vec<(AssetInfo, (PriceSource<OraclePriceSource>, Complexity))>, _> = self
            .assets
            .prefix(self.user)
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
    ) -> AbstractResult<Vec<(AssetEntry, UncheckedPriceSource<OraclePriceSource>)>> {
        let limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(LIST_SIZE_LIMIT) as usize;
        let start_bound = last_asset.as_ref().map(Bound::exclusive);

        let res: Result<Vec<(AssetEntry, UncheckedPriceSource<OraclePriceSource>)>, _> = self
            .sources
            .prefix(self.user)
            .range(deps.storage, start_bound, None, Order::Ascending)
            .take(limit)
            .collect();

        res.map_err(Into::into)
    }

    /// Get the highest complexity present in the oracle
    /// Note: this function will panic in case of missing base asset
    fn highest_complexity(&self, deps: Deps) -> AbstractResult<u8> {
        self.complexity
            .prefix(self.user)
            .keys(deps.storage, None, None, Order::Descending)
            .next()
            // Presence of base asset should be done via `base_asset`
            .unwrap()
            .map_err(Into::into)
    }

    /// get the configuration of an asset
    pub fn asset_config(
        &self,
        deps: Deps,
        asset: &AssetEntry,
    ) -> AbstractResult<UncheckedPriceSource<OraclePriceSource>> {
        self.sources
            .load(deps.storage, (self.user, asset))
            .map_err(Into::into)
    }

    pub fn base_asset(&self, deps: Deps) -> AbstractResult<AssetInfo> {
        let base_asset = self.complexity.may_load(deps.storage, (self.user, 0))?;
        let Some(base_asset) = base_asset else {
            return Err(StdError::generic_err("No base asset registered").into());
        };
        let base_asset_len = base_asset.len();
        if base_asset_len != 1 {
            return Err(StdError::generic_err(format!(
                "{base_asset_len} base assets registered, must be 0 or 1"
            ))
            .into());
        }
        Ok(base_asset[0].clone())
    }
}

#[derive(Default)]
pub struct AssetValue {
    pub base: Uint128,
    pub external: Uint128,
}

impl std::ops::Add for AssetValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        AssetValue {
            base: self.base + rhs.base,
            external: self.base + rhs.base,
        }
    }
}

/// Total value
#[cw_serde]
pub struct TotalValue {
    /// The total value in base denom
    pub total_value: Uint128,
    /// The total value in virtual denom
    pub virtual_total_value: Uint128,
    /// Vec of asset information and their value in the base asset denomination
    pub breakdown: Vec<(AssetInfo, Uint128)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use abstract_testing::prelude::*;
    use cosmwasm_std::{coin, testing::*, Decimal};
    use speculoos::prelude::*;

    use crate::objects::DexAssetPairing;
    type AResult = anyhow::Result<()>;

    pub fn get_ans() -> AnsHost {
        let addr = Addr::unchecked(TEST_ANS_HOST);

        AnsHost::new(addr)
    }

    pub fn base_asset() -> (AssetEntry, UncheckedPriceSource<OraclePriceSource>) {
        (AssetEntry::from(USD), UncheckedPriceSource::None)
    }

    pub fn asset_with_dep() -> (AssetEntry, UncheckedPriceSource<OraclePriceSource>) {
        let asset = AssetEntry::from(EUR);
        let price_source = UncheckedPriceSource::Pair(DexAssetPairing::new(
            AssetEntry::new(EUR),
            AssetEntry::new(USD),
            TEST_DEX,
        ));
        (asset, price_source)
    }

    pub fn asset_as_half() -> (AssetEntry, UncheckedPriceSource<OraclePriceSource>) {
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

        let oracle = Oracle::new("");
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
            .sources
            .prefix("")
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
        let mut oracle = Oracle::new("");

        // add base asset
        oracle.update_assets(deps.as_mut(), &ans, vec![base_asset()], vec![])?;

        let value = oracle.account_value(deps.as_ref())?;
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
    fn query_baseless_account() -> AResult {
        let mut deps = mock_dependencies();
        let mock_ans = MockAnsHost::new().with_defaults();
        deps.querier = mock_ans.to_querier();
        let mut oracle = Oracle::new("");

        let value_result = oracle.account_value(deps.as_ref());

        assert!(value_result.is_err());
        Ok(())
    }

    #[test]
    fn empty_update() -> AResult {
        let mut deps = mock_dependencies();
        let mock_ans = MockAnsHost::new().with_defaults();
        deps.querier = mock_ans.to_querier();
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, vec![coin(1000, USD)]);
        let ans = get_ans();
        let oracle = Oracle::new("");

        // Empty update on empty assets - base asset not found error
        let update_res = oracle.update_assets(deps.as_mut(), &ans, vec![], vec![]);
        assert!(update_res.is_err());

        // add base asset
        oracle.update_assets(deps.as_mut(), &ans, vec![base_asset()], vec![])?;

        // Empty update with assets with assets
        let update_res = oracle.update_assets(deps.as_mut(), &ans, vec![], vec![]);
        assert!(update_res.is_ok());

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
        let mut oracle = Oracle::new("");
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

        let value = oracle.account_value(deps.as_ref())?;
        assert_that!(value.total_value.amount.u128()).is_equal_to(500u128);

        // give the account some base asset
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, vec![coin(1000, USD), coin(1000, EUR)]);

        // assert that the value increases with 1000
        let value = oracle.account_value(deps.as_ref())?;
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

    #[test]
    fn reject_duplicate_entries() -> AResult {
        let mut deps = mock_dependencies();
        let mock_ans = MockAnsHost::new().with_defaults();
        deps.querier = mock_ans.to_querier();
        let ans = get_ans();
        let oracle = Oracle::new("");

        // fails because base asset is not set.
        let res = oracle.update_assets(
            deps.as_mut(),
            &ans,
            vec![asset_as_half()],
            vec![asset_as_half().0],
        );
        assert_that!(res).is_err();
        Ok(())
    }

    // test for pair

    // test for LP tokens

    // test for max complexity
}
