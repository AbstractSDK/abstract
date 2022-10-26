use abstract_os::objects::{AssetEntry, ContractEntry};
use abstract_sdk::MemoryOperation;
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, StdResult, Uint128};
use cw_asset::{Asset, AssetInfo};

use crate::{contract::DexApi, error::DexError};

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;
/// DEX trait resolves asset names and dex to pair and lp address and ensures supported dexes support swaps and liquidity provisioning.
pub trait DEX {
    fn pair_address(
        &self,
        deps: Deps,
        api: &DexApi,
        assets: &mut Vec<&AssetEntry>,
    ) -> StdResult<Addr> {
        let dex_pair = self.pair_contract(assets);
        api.resolve(deps, &dex_pair)
    }
    fn pair_contract(&self, assets: &mut Vec<&AssetEntry>) -> ContractEntry {
        ContractEntry::construct_dex_entry(self.name(), assets)
    }
    fn over_ibc(&self) -> bool;
    fn name(&self) -> &'static str;
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
