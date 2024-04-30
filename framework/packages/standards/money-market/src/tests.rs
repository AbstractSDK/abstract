use std::fmt::Debug;

use cosmwasm_std::{Addr, CosmosMsg, StdError};
use cw_asset::Asset;
use cw_orch::daemon::{live_mock::mock_dependencies, ChainRegistryData as ChainData};

use crate::{MoneyMarketCommand, MoneyMarketError};

pub struct MoneyMarketCommandTester {
    chain: ChainData,
    adapter: Box<dyn MoneyMarketCommand>,
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

impl MoneyMarketCommandTester {
    pub fn new<T: MoneyMarketCommand + 'static>(chain: ChainData, adapter: T) -> Self {
        Self {
            chain,
            adapter: Box::new(adapter),
        }
    }

    pub fn test_deposit(
        &self,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self.adapter.deposit(deps.as_ref(), contract_addr, asset)?;
        Ok(msgs)
    }

    pub fn test_withdraw(
        &self,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self.adapter.withdraw(deps.as_ref(), contract_addr, asset)?;
        Ok(msgs)
    }

    pub fn test_provide_collateral(
        &self,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self
            .adapter
            .provide_collateral(deps.as_ref(), contract_addr, asset)?;
        Ok(msgs)
    }

    pub fn test_withdraw_collateral(
        &self,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self
            .adapter
            .withdraw_collateral(deps.as_ref(), contract_addr, asset)?;
        Ok(msgs)
    }

    pub fn test_borrow(
        &self,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self.adapter.borrow(deps.as_ref(), contract_addr, asset)?;
        Ok(msgs)
    }

    pub fn test_repay(
        &self,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let deps = mock_dependencies(self.chain.clone());
        let msgs = self.adapter.repay(deps.as_ref(), contract_addr, asset)?;
        Ok(msgs)
    }
}
