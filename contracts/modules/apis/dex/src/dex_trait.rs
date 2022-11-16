use abstract_os::objects::{AssetEntry, ContractEntry};
use abstract_sdk::ans_host::AnsHost;
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, StdResult, Uint128};
use cw_asset::{Asset, AssetInfo};

use crate::error::DexError;

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

pub trait Identify {
    fn over_ibc(&self) -> bool;
    fn name(&self) -> &'static str;
}

/// DEX ensures supported dexes support the expected functionality.
/// Trait that implements the actual dex interaction.
pub trait DEX: Identify {
    fn pair_address(
        &self,
        deps: Deps,
        ans_host: &AnsHost,
        assets: &mut Vec<&AssetEntry>,
    ) -> StdResult<Addr> {
        let dex_pair = self.pair_contract(assets);
        ans_host.query_contract(deps, &dex_pair)
    }
    fn pair_contract(&self, assets: &mut Vec<&AssetEntry>) -> ContractEntry {
        ContractEntry::construct_dex_entry(self.name(), assets)
    }
    #[allow(clippy::too_many_arguments)]
    fn swap(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError>;
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
    fn provide_liquidity(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError>;
    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError>;
    // fn raw_swap();
    // fn raw_provide_liquidity();
    fn withdraw_liquidity(
        &self,
        deps: Deps,
        pair_address: Addr,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError>;
    // fn raw_withdraw_liquidity();
    // fn route_swap();
    // fn raw_route_swap();
    fn simulate_swap(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError>;
}
