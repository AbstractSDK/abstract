use std::error::Error;

use abstract_adapter_utils::identity::Identify;
use abstract_core::objects::{DexAssetPairing, PoolAddress, PoolReference};
use abstract_sdk::core::objects::AssetEntry;
use abstract_sdk::feature_objects::AnsHost;
use cosmwasm_std::{CosmosMsg, Decimal, Deps, StdError, Uint128};
use cw_asset::{Asset, AssetInfo};

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

/// # DexCommand
/// ensures DEX adapters support the expected functionality.
///
/// Implements the usual DEX operations.
pub trait DexCommand<E: Error = StdError>: Identify {
    /// Return pool information for given assets pair
    fn pair_address(
        &self,
        deps: Deps,
        ans_host: &AnsHost,
        assets: (AssetEntry, AssetEntry),
    ) -> Result<PoolAddress, StdError> {
        let dex_pair = DexAssetPairing::new(assets.0, assets.1, self.name());
        let mut pool_ref = ans_host
            .query_asset_pairing(&deps.querier, &dex_pair)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        // Currently takes the first pool found, but should be changed to take the best pool
        let found: PoolReference = pool_ref.pop().ok_or(StdError::generic_err(format!(
            "Asset pairing {} not found.",
            dex_pair
        )))?;
        Ok(found.pool_address)
    }

    /// Execute a swap on the given DEX using the swap in question custom logic
    #[allow(clippy::too_many_arguments)]
    fn swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, E>;

    /// Implement your custom swap the DEX
    fn custom_swap(
        &self,
        _deps: Deps,
        _offer_assets: Vec<Asset>,
        _ask_assets: Vec<Asset>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, StdError> {
        // Must be implemented in the base to be available
        Err(StdError::generic_err(format!(
            "Not implemented : {}",
            self.name()
        )))
    }

    /// Provides liquidity on the the DEX
    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, E>;

    /// Provide symmetric liquidity where available depending on the DEX
    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, E>;

    /// Withdraw liquidity from DEX
    fn withdraw_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, E>;

    /// Simulate a swap in the DEX
    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), E>;

    // fn raw_swap();
    // fn raw_provide_liquidity();
    // fn raw_withdraw_liquidity();
    // fn route_swap();
    // fn raw_route_swap();
}
