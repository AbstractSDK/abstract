use abstract_moneymarket_standard::Identify;

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
        coins_in_assets, Fee, FeeOnInput, MoneymarketCommand, MoneymarketError, Return, Spread,
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
        ghost::{
            basic_vault,
            market::{self, PositionResponse},
        },
        KujiraQuerier,
    },
};

#[cfg(feature = "full_integration")]
impl MoneymarketCommand for Kujira {
    fn deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
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
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
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
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
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
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
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
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
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
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
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
    ) -> Result<Decimal, MoneymarketError> {
        // let base_denom = base.to_string();
        // let quote_denom = quote.to_string();

        // let raw_base_price = KujiraQuerier::new(&deps.querier).query_exchange_rate(base_denom)?;
        // // This is how much 1 unit of base is in terms of $
        // let base_price = raw_base_price.normalize(6);

        // let raw_quote_price = KujiraQuerier::new(&deps.querier).query_exchange_rate(base_denom)?;
        // // This is how much 1 unit of quote is in terms of $
        // let quote_price = raw_quote_price.normalize(6);

        // // This is how much 1 unit of base is in terms of quote
        // Ok((base_price / quote_price).inner())

        Ok(Decimal::one())
    }

    fn user_deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError> {
        // We query the xToken balance

        todo!()
    }

    fn user_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = market::QueryMsg::Position { holder: user };

        let query_response: market::PositionResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.collateral_amount)
    }

    fn user_borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = market::QueryMsg::Position { holder: user };

        let query_response: market::PositionResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.debt_shares)
    }

    fn current_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        collateral_asset: AssetInfo,
        borrowed_asset: AssetInfo,
    ) -> Result<Decimal, MoneymarketError> {
        // We get the borrowed_value / collateral value
        let collateral = self.user_collateral(
            deps,
            contract_addr.clone(),
            user.clone(),
            collateral_asset.clone(),
        )?;
        let borrow = self.user_borrow(deps, contract_addr, user, borrowed_asset.clone())?;

        // This represents how much 1 unit of the collateral_asset is worth in terms of the borrowed_asset
        let collateral_price = self.price(deps, collateral_asset, borrowed_asset)?;

        let collateral_value = Decimal::from_ratio(collateral, 1u128) * collateral_price;

        Ok(Decimal::from_ratio(borrow, 1u128) / collateral_value)
    }

    fn max_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneymarketError> {
        let market_msg = market::QueryMsg::Config {};

        let query_response: market::ConfigResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.max_ltv)
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_to_kujira(asset: &Asset) -> Result<kujira::Asset, MoneymarketError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(kujira::Asset {
            amount: asset.amount,
            info: kujira::AssetInfo::NativeToken {
                denom: denom.into(),
            },
        }),
        _ => Err(MoneymarketError::UnsupportedAssetType(
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
