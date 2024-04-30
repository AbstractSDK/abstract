use abstract_adapter::sdk::{
    cw_helpers::Chargeable,
    features::{AbstractNameService, AbstractRegistryAccess},
    Execution,
};
use abstract_money_market_standard::{
    raw_action::MoneyMarketRawAction, MoneyMarketCommand, MoneyMarketError,
};
use cosmwasm_std::{Addr, CosmosMsg, Deps};
use cw_asset::{AssetBase, AssetInfoBase};

use crate::state::MONEY_MARKET_FEES;

pub const DEPOSIT: u64 = 8142;
pub const WITHDRAW: u64 = 8143;
pub const PROVIDE_COLLATERAL: u64 = 8144;
pub const WITHDRAW_COLLATERAL: u64 = 8145;
pub const BORROW: u64 = 8146;
pub const REPAY: u64 = 8147;

impl<T> MoneyMarketAdapter for T where T: AbstractNameService + Execution + AbstractRegistryAccess {}

pub(crate) type ReplyId = u64;

pub trait MoneyMarketAdapter: AbstractNameService + AbstractRegistryAccess + Execution {
    /// resolve the provided money_market action on a local money_market
    fn resolve_money_market_action(
        &self,
        deps: Deps,
        sender: Addr,
        action: MoneyMarketRawAction,
        mut money_market: Box<dyn MoneyMarketCommand>,
    ) -> Result<(Vec<CosmosMsg>, ReplyId), MoneyMarketError> {
        Ok(match action.request {
            abstract_money_market_standard::raw_action::MoneyMarketRawRequest::Deposit { lending_asset } => {
                (self.resolve_deposit(deps, sender, lending_asset, action.contract_addr, money_market.as_mut())?, DEPOSIT)
            }
            abstract_money_market_standard::raw_action::MoneyMarketRawRequest::Withdraw { lent_asset } => {
                (self.resolve_withdraw(deps, sender, lent_asset, action.contract_addr, money_market.as_mut())?, WITHDRAW)
            }
            abstract_money_market_standard::raw_action::MoneyMarketRawRequest::ProvideCollateral { borrowable_asset, collateral_asset } => {
                (self.resolve_provide_collateral(deps, sender, borrowable_asset, collateral_asset, action.contract_addr, money_market.as_mut())?, PROVIDE_COLLATERAL)
            }
            abstract_money_market_standard::raw_action::MoneyMarketRawRequest::WithdrawCollateral { borrowable_asset, collateral_asset } => {
                (self.resolve_withdraw_collateral(deps, sender, borrowable_asset, collateral_asset, action.contract_addr, money_market.as_mut())?, WITHDRAW_COLLATERAL)
            }
            abstract_money_market_standard::raw_action::MoneyMarketRawRequest::Borrow { borrow_asset, collateral_asset } => {
                (self.resolve_borrow(deps, sender, borrow_asset, collateral_asset, action.contract_addr, money_market.as_mut())?, BORROW)
            }
            abstract_money_market_standard::raw_action::MoneyMarketRawRequest::Repay { borrowed_asset, collateral_asset } => {
                (self.resolve_repay(deps, sender, borrowed_asset, collateral_asset, action.contract_addr, money_market.as_mut())?, REPAY)
            }
        })
    }

    fn resolve_deposit(
        &self,
        deps: Deps,
        sender: Addr,
        lending_asset: AssetBase<String>,
        contract_addr: String,
        money_market: &mut dyn MoneyMarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let asset = lending_asset.check(deps.api, None)?;

        money_market.fetch_data(sender, &deps.querier, &self.ans_host(deps)?)?;
        money_market.deposit(deps, contract_addr, asset)
    }

    fn resolve_withdraw(
        &self,
        deps: Deps,
        sender: Addr,
        lending_asset: AssetBase<String>,
        contract_addr: String,
        money_market: &mut dyn MoneyMarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let mut asset = lending_asset.check(deps.api, None)?;

        money_market.fetch_data(sender, &deps.querier, &self.ans_host(deps)?)?;
        let mut withdraw_msgs = money_market.withdraw(deps, contract_addr, asset.clone())?;

        // account for fee
        let dex_fees = MONEY_MARKET_FEES.load(deps.storage)?;
        let fee_msg = asset.charge_usage_fee(dex_fees)?;

        withdraw_msgs.extend(fee_msg);
        Ok(withdraw_msgs)
    }

    fn resolve_provide_collateral(
        &self,
        deps: Deps,
        sender: Addr,
        _borrowed_asset: AssetInfoBase<String>,
        collateral_asset: AssetBase<String>,
        contract_addr: String,
        money_market: &mut dyn MoneyMarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let collateral_asset = collateral_asset.check(deps.api, None)?;

        money_market.fetch_data(sender, &deps.querier, &self.ans_host(deps)?)?;
        money_market.provide_collateral(deps, contract_addr, collateral_asset)
    }

    fn resolve_withdraw_collateral(
        &self,
        deps: Deps,
        sender: Addr,
        _borrowed_asset: AssetInfoBase<String>,
        collateral_asset: AssetBase<String>,
        contract_addr: String,
        money_market: &mut dyn MoneyMarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let collateral_asset = collateral_asset.check(deps.api, None)?;

        money_market.fetch_data(sender, &deps.querier, &self.ans_host(deps)?)?;
        money_market.withdraw_collateral(deps, contract_addr, collateral_asset)
    }

    fn resolve_borrow(
        &self,
        deps: Deps,
        sender: Addr,
        borrowed_asset: AssetBase<String>,
        _collateral_asset: AssetInfoBase<String>,
        contract_addr: String,
        money_market: &mut dyn MoneyMarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let borrowed_asset = borrowed_asset.check(deps.api, None)?;

        money_market.fetch_data(sender, &deps.querier, &self.ans_host(deps)?)?;
        money_market.borrow(deps, contract_addr, borrowed_asset)
    }

    fn resolve_repay(
        &self,
        deps: Deps,
        sender: Addr,
        borrowed_asset: AssetBase<String>,
        _collateral_asset: AssetInfoBase<String>,
        contract_addr: String,
        money_market: &mut dyn MoneyMarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let borrowed_asset = borrowed_asset.check(deps.api, None)?;

        money_market.fetch_data(sender, &deps.querier, &self.ans_host(deps)?)?;
        money_market.repay(deps, contract_addr, borrowed_asset)
    }
}
