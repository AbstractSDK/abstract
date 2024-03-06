use abstract_adapter_utils::identity::Identify;

use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, Uint128};
use cw_asset::{Asset, AssetInfo};

use crate::error::MoneyMarketError;

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

/// # MoneyMarketCommand
/// ensures Money Market adapters support the expected functionality.
///
/// Implements the usual MoneyMarket operations.
pub trait MoneyMarketCommand: Identify {
    /// Deposits funds to be lended on the given Money Market
    fn deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError>;

    /// Withdraw lended funds on the given Money Market
    fn withdraw(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
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
    fn price(
        &self,
        deps: Deps,
        base: AssetInfo,
        quote: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError>;

    fn user_deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
        asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError>;

    fn user_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
        asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError>;

    fn user_borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
        asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError>;

    fn current_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
    ) -> Result<Decimal, MoneyMarketError>;

    fn max_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
    ) -> Result<Decimal, MoneyMarketError>;
}
