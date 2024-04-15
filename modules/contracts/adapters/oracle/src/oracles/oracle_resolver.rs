use abstract_adapter_utils::identity::{
    decompose_platform_name, is_available_on, is_current_chain,
};
use abstract_core::{
    objects::price_source::{AssetConversion, PriceSource},
    AbstractResult,
};
use abstract_oracle_standard::{
    msg::TokensValueResponse,
    state::{AssetValue, Oracle, OraclePriceSource, TotalValue},
    Identify, OracleCommand, OracleError,
};
use cosmwasm_std::{Deps, Env, Uint128};
use cw_asset::Asset;

/// Any oracle should be identified by the adapter
/// This allows erroring the execution before sending any IBC message to another chain
/// This provides superior UX in case of an IBC execution
pub(crate) fn identify_oracle(value: &str) -> Result<Box<dyn Identify>, OracleError> {
    match value {
        #[cfg(feature = "pyth")]
        crate::oracles::pyth::PYTH => Ok(Box::<crate::oracles::pyth::Pyth>::default()),
        _ => Err(OracleError::UnknownProvider(value.to_owned())),
    }
}

pub(crate) fn resolve_oracle(value: &str) -> Result<Box<dyn OracleCommand>, OracleError> {
    match value {
        crate::oracles::pyth::PYTH => Ok(Box::<crate::oracles::pyth::Pyth>::default()),
        _ => Err(OracleError::ForeignOracle(value.to_owned())),
    }
}

/// Given a FULL provider name (e.g. juno>wyndex), returns whether the request is local or over IBC
pub fn is_over_ibc(env: Env, platform_name: &str) -> Result<(String, bool), OracleError> {
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if chain_name.is_some() && !is_current_chain(env.clone(), &chain_name.clone().unwrap()) {
        Ok((local_platform_name, true))
    } else {
        let platform_id = identify_oracle(&local_platform_name)?;
        // We verify the adapter is available on the current chain
        if !is_available_on(platform_id, env, chain_name.as_deref()) {
            return Err(OracleError::UnknownProvider(platform_name.to_string()));
        }
        Ok((local_platform_name, false))
    }
}

pub trait OracleAssetPrice {
    /// Calculates the value of a single asset by recursive conversion to underlying asset(s).
    /// Does not make use of the cache to prevent querying the same price source multiple times.
    fn asset_value(&self, deps: Deps, asset: Asset) -> AbstractResult<AssetValue>;
    fn external_asset_value(&self, price_source: OraclePriceSource) -> AbstractResult<Uint128>;
    /// Calculates the total value of an account's assets by efficiently querying the configured price sources
    ///
    ///
    /// ## Resolve the total value of an account given a base asset.
    /// This process goes as follows
    /// 1. Get the assets for the highest, not visited, complexity.
    /// 2. For each asset query it's balance, get the conversion ratios associated with that asset and load its cached values.
    /// 3. Using the conversion ratio convert the balance and cached values and save the resulting values in the cache for that lower complexity asset.
    /// 4. Repeat until the base asset is reached. (complexity = 0)
    fn account_value(&mut self, deps: Deps) -> AbstractResult<TokensValueResponse>;

    /// Calculates the values of assets for a given complexity level
    fn complexity_value_calculation(
        &mut self,
        deps: Deps,
        complexity: u8,
        account: &str,
    ) -> AbstractResult<TokensValueResponse>;
}

impl OracleAssetPrice for Oracle<'_> {
    fn asset_value(&self, deps: Deps, asset: Asset) -> AbstractResult<AssetValue> {
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

    fn account_value(&mut self, deps: Deps) -> AbstractResult<TokensValueResponse> {
        // fist check that a base asset is registered
        let _ = self.base_asset(deps)?;

        // get the highest complexity
        let start_complexity = self.highest_complexity(deps)?;
        self.complexity_value_calculation(deps, start_complexity, self.user)
    }

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
}
