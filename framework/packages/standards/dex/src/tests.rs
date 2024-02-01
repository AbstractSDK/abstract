use std::fmt::Debug;

use abstract_core::objects::PoolAddress;
use cosmwasm_std::{CosmosMsg, Decimal, StdError};
use cw_asset::{Asset, AssetInfo};
use cw_orch::daemon::{live_mock::mock_dependencies, ChainRegistryData as ChainData};

use crate::{DexCommand, DexError, Fee, FeeOnInput, Return, Spread};

pub struct DexCommandTester {
    chain: ChainData,
    adapter: Box<dyn DexCommand>,
}

pub fn expect_eq<T: PartialEq + Debug>(t1: T, t2: T) -> Result<(), StdError> {
    if t1 != t2 {
        Err(StdError::generic_err(format!(
            "Test failed, wrong result, expected : {:?}, got : {:?}",
            t1, t2
        )))?
    }
    Ok(())
}

impl DexCommandTester {
    pub fn new<T: DexCommand + 'static>(chain: ChainData, adapter: T) -> Self {
        Self {
            chain,
            adapter: Box::new(adapter),
        }
    }

    pub fn test_swap(
        &self,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self.adapter.swap(
            deps.as_ref(),
            pool_id,
            offer_asset,
            ask_asset,
            belief_price,
            max_spread,
        )?;
        Ok(msgs)
    }

    pub fn test_custom_swap(
        &self,
        offer_assets: Vec<Asset>,
        ask_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self
            .adapter
            .custom_swap(deps.as_ref(), offer_assets, ask_assets, max_spread)?;
        Ok(msgs)
    }

    pub fn test_provide_liquidity(
        &self,
        pool_id: PoolAddress,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs =
            self.adapter
                .provide_liquidity(deps.as_ref(), pool_id, offer_assets, max_spread)?;
        Ok(msgs)
    }

    pub fn test_provide_liquidity_symmetric(
        &self,
        pool_id: PoolAddress,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self.adapter.provide_liquidity_symmetric(
            deps.as_ref(),
            pool_id,
            offer_asset,
            paired_assets,
        )?;
        Ok(msgs)
    }

    pub fn test_withdraw_liquidity(
        &self,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self
            .adapter
            .withdraw_liquidity(deps.as_ref(), pool_id, lp_token)?;
        Ok(msgs)
    }

    pub fn test_simulate_swap(
        &self,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let deps = mock_dependencies(self.chain.clone());
        let result = self
            .adapter
            .simulate_swap(deps.as_ref(), pool_id, offer_asset, ask_asset)?;
        Ok(result)
    }
}
