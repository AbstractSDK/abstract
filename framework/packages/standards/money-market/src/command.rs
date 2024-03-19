use abstract_adapter_utils::identity::Identify;

use crate::error::MoneyMarketError;
use abstract_core::objects::{ans_host::AnsHostError, AssetEntry};
use abstract_sdk::feature_objects::AnsHost;
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, QuerierWrapper, Uint128};
use cw_asset::{Asset, AssetInfo};

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

/// # MoneyMarketCommand
/// ensures Money Market adapters support the expected functionality.
///
/// Implements the usual MoneyMarket operations.
pub trait MoneyMarketCommand: Identify {
    fn fetch_data(
        &mut self,
        _addr_as_sender: Addr,
        _querier: &QuerierWrapper,
        _ans_host: &AnsHost,
    ) -> Result<(), MoneyMarketError> {
        Ok(())
    }

    fn lending_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError>;

    fn collateral_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        borrowed_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError>;

    fn borrow_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        borrowed_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError>;

    /// Deposits funds to be lent out on the given Money Market
    fn deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        lending_asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError>;

    /// Withdraw lent funds on the given Money Market
    fn withdraw(
        &self,
        deps: Deps,
        contract_addr: Addr,
        receipt_asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError>;

    /// Provide collateral on the given Money Market
    fn provide_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError>;

    /// Withdraw collateral from the given Money Market
    fn withdraw_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError>;

    /// Borrow funds on the given Money Market
    fn borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError>;

    /// Repay borrowed funds on the given Money Market
    fn repay(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError>;

    //*****************   Queries   ****************/
    // This represents how much 1 unit of the base is worth in terms of the quote
    fn price(
        &self,
        deps: Deps,
        base: AssetInfo,
        quote: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError>;

    fn user_deposit(
        &self,
        deps: Deps,
        lending_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneyMarketError>;

    fn user_collateral(
        &self,
        deps: Deps,
        collateral_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Uint128, MoneyMarketError>;

    fn user_borrow(
        &self,
        deps: Deps,
        borrow_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Uint128, MoneyMarketError>;

    fn current_ltv(
        &self,
        deps: Deps,
        current_ltv_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError>;

    fn current_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        borrowed_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError>;

    fn max_ltv(
        &self,
        deps: Deps,
        max_ltv_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError>;

    fn max_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError>;
}
