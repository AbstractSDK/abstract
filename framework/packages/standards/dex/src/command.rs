use abstract_adapter_utils::identity::Identify;
use abstract_sdk::feature_objects::{AnsHost, RegistryContract};
use abstract_std::objects::{AssetEntry, DexAssetPairing, PoolAddress, PoolReference};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, Uint128};
use cw_asset::{Asset, AssetInfo};

use crate::{error::DexError, msg::SwapNode};

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

/// # DexCommand
/// ensures DEX adapters support the expected functionality.
///
/// Implements the usual DEX operations.
pub trait DexCommand: Identify {
    /// Return pool information for given assets pair
    fn pool_reference(
        &self,
        deps: Deps,
        ans_host: &AnsHost,
        assets: (AssetEntry, AssetEntry),
    ) -> Result<PoolReference, DexError> {
        let dex_pair = DexAssetPairing::new(assets.0, assets.1, self.name());
        let mut pool_ref = ans_host.query_asset_pairing(&deps.querier, &dex_pair)?;
        // Currently takes the first pool found, but should be changed to take the best pool
        let found: PoolReference = pool_ref.pop().ok_or(DexError::AssetPairingNotFound {
            asset_pairing: dex_pair,
        })?;
        Ok(found)
    }

    /// Return pool address for given assets pair
    fn pair_address(
        &self,
        deps: Deps,
        ans_host: &AnsHost,
        assets: (AssetEntry, AssetEntry),
    ) -> Result<PoolAddress, DexError> {
        Ok(self.pool_reference(deps, ans_host, assets)?.pool_address)
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

    /// Execute a swap on the given DEX using the swap in question custom logic
    #[allow(clippy::too_many_arguments)]
    #[allow(unused)]
    // TODO: do we need normal swap or should every integration be swap_route?
    fn swap_route(
        &self,
        deps: Deps,
        swap_route: Vec<SwapNode<Addr>>,
        offer_asset: Asset,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        unimplemented!();
    }

    /// Provides liquidity on the the DEX
    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
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

    /// Fetch data for execute methods
    fn fetch_data(
        &mut self,
        _deps: Deps,
        _addr_as_sender: Addr,
        _registry_contract: RegistryContract,
        _ans_host: AnsHost,
    ) -> Result<(), DexError> {
        // Dummy implementation, since most of dexes does not require this method
        Ok(())
    }
}
