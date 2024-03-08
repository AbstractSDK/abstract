use abstract_core::objects::pool_id::PoolAddressBase;
use abstract_moneymarket_standard::{
    raw_action::MoneymarketRawAction, MoneymarketCommand, MoneymarketError,
};
use abstract_sdk::{
    cw_helpers::Chargeable,
    features::{AbstractNameService, AbstractRegistryAccess},
    Execution,
};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps};
use cw_asset::{AssetBase, AssetInfoBase};

use crate::state::MONEYMARKET_FEES;

pub const DEPOSIT: u64 = 8142;
pub const WITHDRAW: u64 = 8143;
pub const PROVIDE_COLLATERAL: u64 = 8144;
pub const WITHDRAW_COLLATERAL: u64 = 8145;
pub const BORROW: u64 = 8146;
pub const REPAY: u64 = 8147;

impl<T> MoneymarketAdapter for T where T: AbstractNameService + Execution + AbstractRegistryAccess {}

pub(crate) type ReplyId = u64;

pub trait MoneymarketAdapter: AbstractNameService + AbstractRegistryAccess + Execution {
    /// resolve the provided moneymarket action on a local moneymarket
    fn resolve_moneymarket_action(
        &self,
        deps: Deps,
        sender: Addr,
        action: MoneymarketRawAction,
        mut moneymarket: Box<dyn MoneymarketCommand>,
    ) -> Result<(Vec<CosmosMsg>, ReplyId), MoneymarketError> {
        Ok(match action.request {
            abstract_moneymarket_standard::raw_action::MoneymarketRawRequest::Deposit { asset } => {
                (self.resolve_deposit(deps, sender, asset, action.contract_addr, moneymarket.as_mut())?, DEPOSIT)
            }
            abstract_moneymarket_standard::raw_action::MoneymarketRawRequest::Withdraw { asset } => {
                (self.resolve_withdraw(deps, sender, asset, action.contract_addr, moneymarket.as_mut())?, WITHDRAW)
            }
            abstract_moneymarket_standard::raw_action::MoneymarketRawRequest::ProvideCollateral { borrowed_asset, collateral_asset } => {
                (self.resolve_provide_collateral(deps, sender, borrowed_asset, collateral_asset, action.contract_addr, moneymarket.as_mut())?, PROVIDE_COLLATERAL)
            }
            abstract_moneymarket_standard::raw_action::MoneymarketRawRequest::WithdrawCollateral { borrowed_asset, collateral_asset } => {
                (self.resolve_withdraw_collateral(deps, sender, borrowed_asset, collateral_asset, action.contract_addr, moneymarket.as_mut())?, WITHDRAW_COLLATERAL)
            }
            abstract_moneymarket_standard::raw_action::MoneymarketRawRequest::Borrow { borrowed_asset, collateral_asset } => {
                (self.resolve_borrow(deps, sender, borrowed_asset, collateral_asset, action.contract_addr, moneymarket.as_mut())?, BORROW)
            }
            abstract_moneymarket_standard::raw_action::MoneymarketRawRequest::Repay { borrowed_asset, collateral_asset } => {
                (self.resolve_repay(deps, sender, borrowed_asset, collateral_asset, action.contract_addr, moneymarket.as_mut())?, REPAY)
            }
        })
    }

    fn resolve_deposit(
        &self,
        deps: Deps,
        sender: Addr,
        asset: AssetBase<String>,
        contract_addr: String,
        moneymarket: &mut dyn MoneymarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let asset = asset.check(deps.api, None)?;

        moneymarket.deposit(deps, contract_addr, asset)
    }

    fn resolve_withdraw(
        &self,
        deps: Deps,
        sender: Addr,
        asset: AssetBase<String>,
        contract_addr: String,
        moneymarket: &mut dyn MoneymarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let asset = asset.check(deps.api, None)?;

        moneymarket.withdraw(deps, contract_addr, asset)
    }

    fn resolve_provide_collateral(
        &self,
        deps: Deps,
        sender: Addr,
        borrowed_asset: AssetInfoBase<String>,
        collateral_asset: AssetBase<String>,
        contract_addr: String,
        moneymarket: &mut dyn MoneymarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let borrowed_asset = borrowed_asset.check(deps.api, None)?;
        let collateral_asset = collateral_asset.check(deps.api, None)?;

        moneymarket.provide_collateral(deps, contract_addr, collateral_asset)
    }

    fn resolve_withdraw_collateral(
        &self,
        deps: Deps,
        sender: Addr,
        borrowed_asset: AssetInfoBase<String>,
        collateral_asset: AssetBase<String>,
        contract_addr: String,
        moneymarket: &mut dyn MoneymarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let borrowed_asset = borrowed_asset.check(deps.api, None)?;
        let collateral_asset = collateral_asset.check(deps.api, None)?;

        moneymarket.withdraw_collateral(deps, contract_addr, collateral_asset)
    }

    fn resolve_borrow(
        &self,
        deps: Deps,
        sender: Addr,
        borrowed_asset: AssetBase<String>,
        collateral_asset: AssetInfoBase<String>,
        contract_addr: String,
        moneymarket: &mut dyn MoneymarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let borrowed_asset = borrowed_asset.check(deps.api, None)?;
        let collateral_asset = collateral_asset.check(deps.api, None)?;

        moneymarket.borrow(deps, contract_addr, borrowed_asset)
    }

    fn resolve_repay(
        &self,
        deps: Deps,
        sender: Addr,
        borrowed_asset: AssetBase<String>,
        collateral_asset: AssetInfoBase<String>,
        contract_addr: String,
        moneymarket: &mut dyn MoneymarketCommand,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let contract_addr = deps.api.addr_validate(&contract_addr)?;
        let borrowed_asset = borrowed_asset.check(deps.api, None)?;
        let collateral_asset = collateral_asset.check(deps.api, None)?;

        moneymarket.repay(deps, contract_addr, borrowed_asset)
    }
}
