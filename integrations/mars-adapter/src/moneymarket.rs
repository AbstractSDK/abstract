use abstract_moneymarket_standard::Identify;
use abstract_sdk::{
    core::objects::{ans_host::AnsHostError, AssetEntry, ContractEntry},
    feature_objects::AnsHost,
};
use cosmwasm_std::QuerierWrapper;

use crate::{AVAILABLE_CHAINS, MARS};

// Source https://docs.rs/kujira/0.8.2/kujira/
#[derive(Default)]
pub struct Mars {
    pub oracle_contract: Option<Addr>,
}

impl Identify for Mars {
    fn name(&self) -> &'static str {
        MARS
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use {
    abstract_moneymarket_standard::{MoneymarketCommand, MoneymarketError},
    cosmwasm_std::{wasm_execute, Addr, CosmosMsg, Decimal, Deps, Uint128},
    cw_asset::{Asset, AssetInfo},
};

#[cfg(feature = "full_integration")]
impl MoneymarketCommand for Mars {
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
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Deposit { on_behalf_of: None };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        lending_asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Withdraw {
            recipient: None,
            denom: lending_asset.to_string(),
            amount: Some(lending_asset.amount),
        };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn provide_collateral(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Deposit { on_behalf_of: None };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw_collateral(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Withdraw {
            recipient: None,
            denom: asset.to_string(),
            amount: Some(asset.amount),
        };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn borrow(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Borrow {
            recipient: None,
            denom: asset.to_string(),
            amount: asset.amount,
        };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

        Ok(vec![msg.into()])
    }

    fn repay(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Repay { on_behalf_of: None };

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
        _borrowed_asset: AssetInfo, // borrowed asset is not needed inside mars
        collateral_asset: AssetInfo,
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserCollateral {
            user: user.to_string(),
            denom: collateral_asset.to_string(),
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
        borrowed_asset: AssetInfo,
        _collateral_asset: AssetInfo, // collateral asset is not needed for borrow in mars implementation
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserDebt {
            user: user.to_string(),
            denom: borrowed_asset.to_string(),
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
        _borrowed_asset: AssetInfo, // LTV is global on Mars and doesn't depend on collateral asset
        _collateral_asset: AssetInfo, // LTV is global on Mars and doesn't depend on borrowed asset
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
        _borrowed_asset: AssetInfo, // LTV is global on Mars and doesn't depend on borrowing asset
        _collateral_asset: AssetInfo, // LTV is global on Mars and doesn't depend on collateral asset
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

    fn max_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
        _collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.red_bank(querier, ans_host)
    }

    fn current_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        _lending_asset: AssetEntry,
        _collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.red_bank(querier, ans_host)
    }
}

#[cfg(feature = "full_integration")]
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
