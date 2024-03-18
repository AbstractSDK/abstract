use abstract_money_market_standard::Identify;

use cosmwasm_std::Addr;

use crate::{AVAILABLE_CHAINS, CAVERN};

#[derive(Default)]
pub struct Cavern {
    pub oracle_contract: Option<Addr>,
}

impl Identify for Cavern {
    fn name(&self) -> &'static str {
        CAVERN
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use {
    abstract_money_market_standard::{MoneyMarketCommand, MoneyMarketError},
    abstract_sdk::{
        core::objects::{ans_host::AnsHostError, AssetEntry, ContractEntry},
        feature_objects::AnsHost,
    },
    cosmwasm_std::{to_json_binary, QuerierWrapper, StdError},
    cosmwasm_std::{wasm_execute, CosmosMsg, Decimal, Deps, Uint128},
    cw_asset::{Asset, AssetInfo},
};

#[cfg(feature = "full_integration")]
impl MoneyMarketCommand for Cavern {
    fn fetch_data(
        &mut self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> Result<(), MoneyMarketError> {
        let contract_entry = ContractEntry {
            protocol: self.name().to_string(),
            contract: "oracle".to_string(),
        };

        self.oracle_contract = Some(ans_host.query_contract(querier, &contract_entry)?);

        Ok(())
    }

    fn deposit(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = moneymarket::market::ExecuteMsg::DepositStable {};

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw(
        &self,
        deps: Deps,
        contract_addr: Addr,
        lending_asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let aterra_address = self.aterra_address(&deps.querier, contract_addr.clone())?;

        let state = self.market_state(&deps.querier, contract_addr.clone())?;

        let decimal_exchange_rate: Decimal = state.prev_exchange_rate.try_into()?;
        let withdraw_amount = (Decimal::from_ratio(lending_asset.amount, 1u128)
            / decimal_exchange_rate)
            * Uint128::one();

        let vault_msg = moneymarket::market::Cw20HookMsg::RedeemStable {};

        let msg = Asset::cw20(aterra_address, withdraw_amount)
            .send_msg(contract_addr.clone(), to_json_binary(&vault_msg)?)?;

        Ok(vec![msg])
    }

    fn provide_collateral(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = moneymarket::custody::Cw20HookMsg::DepositCollateral { borrower: None };

        let msg = asset.send_msg(contract_addr, to_json_binary(&vault_msg)?)?;

        Ok(vec![msg])
    }

    fn withdraw_collateral(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = moneymarket::custody::ExecuteMsg::WithdrawCollateral {
            amount: Some(asset.amount.into()),
        };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn borrow(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = moneymarket::market::ExecuteMsg::BorrowStable {
            borrow_amount: asset.amount.into(),
            to: None,
        };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn repay(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = moneymarket::market::ExecuteMsg::RepayStable { borrower: None };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn price(
        &self,
        deps: Deps,
        base: AssetInfo,
        quote: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        let oracle_contract = &self.oracle_contract.clone().unwrap();
        let price: moneymarket::oracle::PriceResponse = deps.querier.query_wasm_smart(
            oracle_contract,
            &moneymarket::oracle::QueryMsg::Price {
                base: base.to_string(),
                quote: quote.to_string(),
            },
        )?;

        Ok(price.rate.try_into()?)
    }

    fn user_deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        _asset: AssetInfo, // contract_addr is already lending asset specific
    ) -> Result<Uint128, MoneyMarketError> {
        let aterra_address = self.aterra_address(&deps.querier, contract_addr.clone())?;

        let state = self.market_state(&deps.querier, contract_addr.clone())?;

        let raw_atoken_balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
            aterra_address,
            &cw20_base::msg::QueryMsg::Balance {
                address: user.to_string(),
            },
        )?;

        let decimal_exchange_rate: Decimal = state.prev_exchange_rate.try_into()?;

        Ok(raw_atoken_balance.balance * decimal_exchange_rate)
    }

    fn user_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        _borrowed_asset: AssetInfo, // contract_addr is already borrowed asset specific
        _collateral_asset: AssetInfo, // contract_addr is already collateral asset specific
    ) -> Result<Uint128, MoneyMarketError> {
        let custody_msg = moneymarket::custody::QueryMsg::Borrower {
            address: user.to_string(),
        };

        let query_response: moneymarket::custody::BorrowerResponse =
            deps.querier.query_wasm_smart(contract_addr, &custody_msg)?;

        Ok(query_response.balance.try_into()?)
    }

    fn user_borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        _borrowed_asset: AssetInfo, // contract_addr is already borrowed asset specific
        _collateral_asset: AssetInfo, // contract_addr is already collateral asset specific
    ) -> Result<Uint128, MoneyMarketError> {
        let market_msg = moneymarket::market::QueryMsg::BorrowerInfo {
            borrower: user.to_string(),
            block_height: None,
        };

        let query_response: moneymarket::market::BorrowerInfoResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.loan_amount.try_into()?)
    }

    /// Current loan amount is located in the market contract
    /// Current collateral amounts are located in the custody contracts
    fn current_ltv(
        &self,
        deps: Deps,
        overseer_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        let borrow_limit: moneymarket::overseer::BorrowLimitResponse =
            deps.querier.query_wasm_smart(
                overseer_addr.clone(),
                &moneymarket::overseer::QueryMsg::BorrowLimit {
                    borrower: user.to_string(),
                    block_time: None,
                },
            )?;

        let overseer_config: moneymarket::overseer::ConfigResponse = deps
            .querier
            .query_wasm_smart(overseer_addr, &moneymarket::overseer::QueryMsg::Config {})?;

        let current_borrow = self.user_borrow(
            deps,
            deps.api.addr_validate(&overseer_config.market_contract)?,
            user,
            borrowed_asset,
            collateral_asset,
        )?;

        let borrow_limit: Uint128 = borrow_limit.borrow_limit.try_into()?;
        Ok(Decimal::from_ratio(current_borrow, borrow_limit))
    }

    /// This info is located inside the overseer contract (whitelist query) and only inside there
    fn max_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        _user: Addr,                // The max LTV doesn't depend on the user in Cavern
        _borrowed_asset: AssetInfo, // The max LTV doesn't depend on the borrowed asset in Cavern
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        let overseer_msg = moneymarket::overseer::QueryMsg::Whitelist {
            collateral_token: Some(collateral_asset.to_string()),
            start_after: None,
            limit: None,
        };

        let query_response: moneymarket::overseer::WhitelistResponse = deps
            .querier
            .query_wasm_smart(contract_addr, &overseer_msg)?;

        Ok(query_response.elems[0].max_ltv.try_into()?)
    }

    fn current_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
        _collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.overseer_address(querier, ans_host)
    }

    fn max_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
        _collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.overseer_address(querier, ans_host)
    }

    fn lending_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        let lending_contract = ContractEntry {
            protocol: self.name().to_string(),
            contract: "market".to_string(),
        };

        ans_host.query_contract(querier, &lending_contract)
    }

    fn collateral_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        let lending_contract = ContractEntry {
            protocol: self.name().to_string(),
            contract: format!("custody/{}", collateral_asset),
        };

        ans_host.query_contract(querier, &lending_contract)
    }

    fn borrow_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
        _collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        let lending_contract = ContractEntry {
            protocol: self.name().to_string(),
            contract: "market".to_string(),
        };

        ans_host.query_contract(querier, &lending_contract)
    }
}

#[cfg(feature = "full_integration")]
impl Cavern {
    fn overseer_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> Result<Addr, AnsHostError> {
        let lending_contract = ContractEntry {
            protocol: self.name().to_string(),
            contract: "overseer".to_string(),
        };

        ans_host.query_contract(querier, &lending_contract)
    }
    fn aterra_address(
        &self,
        querier: &QuerierWrapper,
        market_contract: Addr,
    ) -> Result<Addr, MoneyMarketError> {
        let config: moneymarket::market::ConfigResponse =
            querier.query_wasm_smart(market_contract, &moneymarket::market::QueryMsg::Config {})?;

        Ok(Addr::unchecked(config.aterra_contract))
    }
    fn market_state(
        &self,
        querier: &QuerierWrapper,
        market_contract: Addr,
    ) -> Result<moneymarket::market::StateResponse, StdError> {
        querier.query_wasm_smart(
            market_contract.clone(),
            &moneymarket::market::QueryMsg::State { block_height: None },
        )
    }
}