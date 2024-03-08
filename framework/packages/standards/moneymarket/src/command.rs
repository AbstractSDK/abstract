use abstract_adapter_utils::identity::Identify;

use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, Uint128};
use cw_asset::{Asset, AssetInfo};

use crate::error::MoneymarketError;

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

/// # MoneymarketCommand
/// ensures Money Market adapters support the expected functionality.
///
/// Implements the usual Moneymarket operations.
pub trait MoneymarketCommand: Identify {
    /// Deposits funds to be lended on the given Money Market
    fn deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError>;

    /// Withdraw lended funds on the given Money Market
    fn withdraw(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
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
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError>;

    fn user_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError>;

    fn user_borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError>;

    fn current_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        collateral_asset: AssetInfo,
        borrowed_asset: AssetInfo,
    ) -> Result<Decimal, MoneymarketError>;

    fn max_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneymarketError>;
}
