use crate::error::DexError;
use abstract_core::objects::{DexAssetPairing, PoolAddress, PoolReference};
use abstract_sdk::core::objects::AssetEntry;
use abstract_sdk::feature_objects::AnsHost;
use cosmwasm_std::{CosmosMsg, Decimal, Deps, Uint128};
use cw_asset::{Asset, AssetInfo};

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

pub trait Identify {
    fn over_ibc(&self) -> bool;
    fn name(&self) -> &'static str;
}

/// # DexCommand
/// ensures DEX adapters support the expected functionality.
///
/// Implements the usual DEX operations.
pub trait DexCommand: Identify {
    /// Return pool information for given assets pair
    fn pair_address(
        &self,
        deps: Deps,
        ans_host: &AnsHost,
        assets: (AssetEntry, AssetEntry),
    ) -> Result<PoolAddress, DexError> {
        let dex_pair = DexAssetPairing::new(assets.0, assets.1, self.name());
        let mut pool_ref = ans_host.query_asset_pairing(&deps.querier, &dex_pair)?;
        // Currently takes the first pool found, but should be changed to take the best pool
        let found: PoolReference = pool_ref.pop().ok_or(DexError::AssetPairingNotFound {
            asset_pairing: dex_pair,
        })?;
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
    ) -> Result<Vec<CosmosMsg>, DexError>;

    /// Implement your custom swap the DEX
    fn custom_swap(
        &self,
        _deps: Deps,
        _offer_assets: Vec<Asset>,
        _ask_assets: Vec<Asset>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        // Must be implemented in the base to be available
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    /// Provides liquidity on the the DEX
    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError>;

    /// Provide symmetric liquidity where available depending on the DEX
    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError>;

    /// Withdraw liquidity from DEX
    fn withdraw_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError>;

    /// Simulate a swap in the DEX
    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError>;

    // fn raw_swap();
    // fn raw_provide_liquidity();
    // fn raw_withdraw_liquidity();
    // fn route_swap();
    // fn raw_route_swap();
}
