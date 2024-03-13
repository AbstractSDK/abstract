use abstract_moneymarket_standard::Identify;
use abstract_sdk::{
    core::objects::{ans_host::AnsHostError, AnsAsset, AssetEntry, ContractEntry},
    feature_objects::AnsHost,
};
use cosmwasm_std::{to_json_binary, QuerierWrapper};

use crate::{AVAILABLE_CHAINS, MARS};

// Source https://docs.rs/kujira/0.8.2/kujira/
#[derive(Default)]
pub struct Cavern {
    pub oracle_contract: Option<Addr>,
}

impl Identify for Cavern {
    fn name(&self) -> &'static str {
        MARS
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use {
    abstract_moneymarket_standard::{
        coins_in_assets, Fee, FeeOnInput, MoneymarketCommand, MoneymarketError, Return, Spread,
    },
    abstract_sdk::core::objects::PoolAddress,
    cosmwasm_std::{
        wasm_execute, Addr, Coin, CosmosMsg, Decimal, Decimal256, Deps, StdError, StdResult,
        Uint128,
    },
    cw_asset::{Asset, AssetInfo, AssetInfoBase},
};

#[cfg(feature = "full_integration")]
impl MoneymarketCommand for Cavern {
    fn fetch_data(
        &mut self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> Result<(), MoneymarketError> {
        let contract_entry = ContractEntry {
            protocol: self.name().to_string(),
            contract: "oracle".to_string(),
        };

        self.oracle_contract = Some(ans_host.query_contract(querier, &contract_entry)?);

        Ok(())
    }

    fn deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = moneymarket::market::ExecuteMsg::DepositStable {};

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw(
        &self,
        deps: Deps,
        contract_addr: Addr,
        receipt_asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = moneymarket::market::Cw20HookMsg::RedeemStable {};

        receipt_asset.send_msg(contract_addr, to_json_binary(&vault_msg)?)?;

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn provide_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = moneymarket::custody::Cw20HookMsg::DepositCollateral { borrower: None };

        asset.send_msg(contract_addr, to_json_binary(&vault_msg)?)?;

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn withdraw_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = moneymarket::custody::ExecuteMsg::WithdrawCollateral {
            amount: Some(asset.amount.into()),
        };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = moneymarket::market::ExecuteMsg::BorrowStable {
            borrow_amount: asset.amount.into(),
            to: None,
        };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn repay(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = moneymarket::market::ExecuteMsg::RepayStable { borrower: None };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn price(
        &self,
        deps: Deps,
        base: AssetInfo,
        quote: AssetInfo,
    ) -> Result<Decimal, MoneymarketError> {
        let oracle_contract = &self.oracle_contract.clone().unwrap();
        let base_price: mars_red_bank_types::oracle::PriceResponse =
            deps.querier.query_wasm_smart(
                oracle_contract,
                &mars_red_bank_types::oracle::QueryMsg::Price {
                    denom: base.to_string(),
                },
            )?;
        let quote_price: mars_red_bank_types::oracle::PriceResponse =
            deps.querier.query_wasm_smart(
                oracle_contract,
                &mars_red_bank_types::oracle::QueryMsg::Price {
                    denom: quote.to_string(),
                },
            )?;

        Ok(base_price.price / quote_price.price)
    }

    fn user_deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserCollateral {
            user: user.to_string(),
            denom: asset.to_string(),
        };

        let query_response: mars_red_bank_types::red_bank::UserCollateralResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.amount_scaled)
    }

    fn user_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserCollateral {
            user: user.to_string(),
            denom: asset.to_string(),
        };

        let query_response: mars_red_bank_types::red_bank::UserCollateralResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.amount_scaled)
    }

    fn user_borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserDebt {
            user: user.to_string(),
            denom: asset.to_string(),
        };

        let query_response: mars_red_bank_types::red_bank::UserDebtResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.amount_scaled)
    }

    fn current_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        collateral_asset: AssetInfo,
        borrowed_asset: AssetInfo,
    ) -> Result<Decimal, MoneymarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserPosition {
            user: user.to_string(),
        };

        let query_response: mars_red_bank_types::red_bank::UserPositionResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(Decimal::from_ratio(
            query_response.total_collateralized_debt,
            query_response.total_enabled_collateral,
        ))
    }

    fn max_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneymarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserPosition {
            user: user.to_string(),
        };

        let query_response: mars_red_bank_types::red_bank::UserPositionResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(Decimal::from_ratio(
            query_response.weighted_max_ltv_collateral,
            query_response.total_enabled_collateral,
        ))
    }

    fn lending_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.red_bank(querier, ans_host)
    }

    /// For Mars, there is no receipt asset, however, we need to pass the denom to the contract call, so this is warranted here
    fn lending_receipt_asset(
        &self,
        _querier: &QuerierWrapper,
        _ans_host: &AnsHost,
        lending_asset: AssetEntry,
    ) -> Result<AssetEntry, AnsHostError> {
        Ok(lending_asset)
    }

    fn collateral_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
        _collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.red_bank(querier, ans_host)
    }

    fn borrow_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
        _collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.red_bank(querier, ans_host)
    }
}

impl Mars {
    fn red_bank(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> Result<Addr, AnsHostError> {
        let contract_entry = ContractEntry {
            protocol: self.name().to_string(),
            contract: "red-bank".to_string(),
        };

        ans_host
            .query_contract(querier, &contract_entry)
            .map_err(Into::into)
    }
}
