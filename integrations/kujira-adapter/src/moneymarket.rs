use abstract_moneymarket_standard::Identify;
use kujira::ghost::{basic_vault, market};

use crate::{AVAILABLE_CHAINS, KUJIRA};

// Source https://docs.rs/kujira/0.8.2/kujira/
#[derive(Default)]
pub struct Kujira {}

impl Identify for Kujira {
    fn name(&self) -> &'static str {
        KUJIRA
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use ::{
    abstract_moneymarket_standard::{
        coins_in_assets, Fee, FeeOnInput, MoneyMarketCommand, MoneyMarketError, Return, Spread,
    },
    abstract_sdk::core::objects::PoolAddress,
    cosmwasm_std::{
        wasm_execute, Addr, Coin, CosmosMsg, Decimal, Decimal256, Deps, StdError, StdResult,
        Uint128,
    },
    cw_asset::{Asset, AssetInfo, AssetInfoBase},
    kujira::{
        bow::{
            self,
            market_maker::{ConfigResponse, PoolResponse},
        },
        fin,
    },
};

#[cfg(feature = "full_integration")]
impl MoneyMarketCommand for Kujira {
    fn deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg =
            basic_vault::ExecuteMsg::Deposit(basic_vault::DepositMsg { callback: None });

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = basic_vault::ExecuteMsg::Withdraw(basic_vault::WithdrawMsg {
            callback: None,
            amount: asset.amount,
        });

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn provide_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = market::ExecuteMsg::Deposit(market::DepositMsg {
            position_holder: None,
        });

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = market::ExecuteMsg::Withdraw(market::WithdrawMsg {
            amount: asset.amount,
            withdraw_to: None,
        });

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = market::ExecuteMsg::Borrow(market::BorrowMsg {
            amount: asset.amount,
        });

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn repay(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = market::ExecuteMsg::Repay(market::RepayMsg {
            position_holder: None,
        });

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn price(
        &self,
        deps: Deps,
        base: AssetInfo,
        quote: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        todo!()
    }

    fn user_deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
        asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        todo!()
    }

    fn user_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
        asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        todo!()
    }

    fn user_borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
        asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        todo!()
    }

    fn current_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
    ) -> Result<Decimal, MoneyMarketError> {
        todo!()
    }

    fn max_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: String,
    ) -> Result<Decimal, MoneyMarketError> {
        todo!()
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_to_kujira(asset: &Asset) -> Result<kujira::Asset, MoneyMarketError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(kujira::Asset {
            amount: asset.amount,
            info: kujira::AssetInfo::NativeToken {
                denom: denom.into(),
            },
        }),
        _ => Err(MoneyMarketError::UnsupportedAssetType(
            asset.info.to_string(),
        )),
    }
}

#[cfg(feature = "full_integration")]
/// Converts [`Decimal`] to [`Decimal256`].
pub fn decimal2decimal256(dec_value: Decimal) -> StdResult<Decimal256> {
    Decimal256::from_atomics(dec_value.atomics(), dec_value.decimal_places()).map_err(|_| {
        StdError::generic_err(format!(
            "Failed to convert Decimal {} to Decimal256",
            dec_value
        ))
    })
}
