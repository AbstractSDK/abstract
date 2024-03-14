use abstract_adapter_utils::identity::Identify;

use crate::error::MoneymarketError;
use abstract_core::objects::{ans_host::AnsHostError, AssetEntry};
use abstract_sdk::feature_objects::AnsHost;
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, QuerierWrapper, Uint128};
use cw_asset::{Asset, AssetInfo};

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

/// # MoneymarketCommand
/// ensures Money Market adapters support the expected functionality.
///
/// Implements the usual Moneymarket operations.
pub trait MoneymarketCommand: Identify {
    fn fetch_data(
        &mut self,
        _querier: &QuerierWrapper,
        _ans_host: &AnsHost,
    ) -> Result<(), MoneymarketError> {
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

    /// Deposits funds to be lended on the given Money Market
    fn deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        lending_asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError>;

    /// Withdraw lended funds on the given Money Market
    fn withdraw(
        &self,
        deps: Deps,
        contract_addr: Addr,
        receipt_asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError>;

    /// Provide collateral on the given Money Market
    fn provide_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError>;

    /// Withdraw collateral from the given Money Market
    fn withdraw_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError>;

    /// Borrow funds on the given Money Market
    fn borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError>;

    /// Repay borrowed funds on the given Money Market
    fn repay(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError>;

    //*****************   Queries   ****************/
    // This represents how much 1 unit of the base is worth in terms of the quote
    fn price(
        &self,
        deps: Deps,
        base: AssetInfo,
        quote: AssetInfo,
    ) -> Result<Decimal, MoneymarketError>;

    fn user_deposit(
        &self,
        deps: Deps,
        lending_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError>;

    fn user_collateral(
        &self,
        deps: Deps,
        collateral_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError>;

    fn user_borrow(
        &self,
        deps: Deps,
        borrow_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError>;

    fn current_ltv(
        &self,
        deps: Deps,
        current_ltv_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneymarketError>;

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
    ) -> Result<Decimal, MoneymarketError>;

    fn max_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError>;
}
