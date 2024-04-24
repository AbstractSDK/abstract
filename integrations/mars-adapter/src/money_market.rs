use abstract_money_market_standard::Identify;

use cosmwasm_std::Addr;

use crate::{AVAILABLE_CHAINS, MARS};

// https://docs.marsprotocol.io/docs/develop/contracts/red-bank
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
    abstract_money_market_standard::{MoneyMarketCommand, MoneyMarketError},
    abstract_sdk::{
        feature_objects::AnsHost,
        std::objects::{ans_host::AnsHostError, AssetEntry, ContractEntry},
    },
    cosmwasm_std::{wasm_execute, CosmosMsg, Decimal, Deps, QuerierWrapper, Uint128},
    cw_asset::{Asset, AssetInfo},
};

#[cfg(feature = "full_integration")]
impl MoneyMarketCommand for Mars {
    fn fetch_data(
        &mut self,
        _addr_as_sender: Addr,
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
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Deposit { on_behalf_of: None };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        lending_asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let denom = unwrap_native(lending_asset.info)?;

        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Withdraw {
            recipient: None,
            denom,
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
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Deposit { on_behalf_of: None };

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw_collateral(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Withdraw {
            recipient: None,
            denom: unwrap_native(asset.info)?,
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
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Borrow {
            recipient: None,
            denom: unwrap_native(asset.info)?,
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
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let vault_msg = mars_red_bank_types::red_bank::ExecuteMsg::Repay { on_behalf_of: None };

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
        let base_price: mars_red_bank_types::oracle::PriceResponse =
            deps.querier.query_wasm_smart(
                oracle_contract,
                &mars_red_bank_types::oracle::QueryMsg::Price {
                    denom: unwrap_native(base)?,
                },
            )?;
        let quote_price: mars_red_bank_types::oracle::PriceResponse =
            deps.querier.query_wasm_smart(
                oracle_contract,
                &mars_red_bank_types::oracle::QueryMsg::Price {
                    denom: unwrap_native(quote)?,
                },
            )?;

        Ok(base_price.price.checked_div(quote_price.price)?)
    }

    fn user_deposit(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        asset: AssetInfo,
    ) -> Result<Uint128, MoneyMarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserCollateral {
            user: user.to_string(),
            denom: unwrap_native(asset)?,
        };

        let query_response: mars_red_bank_types::red_bank::UserCollateralResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.amount)
    }

    fn user_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        _borrowed_asset: AssetInfo, // borrowed asset is not needed inside mars
        collateral_asset: AssetInfo,
    ) -> Result<Uint128, MoneyMarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserCollateral {
            user: user.to_string(),
            denom: unwrap_native(collateral_asset)?,
        };

        let query_response: mars_red_bank_types::red_bank::UserCollateralResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.amount)
    }

    fn user_borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        _collateral_asset: AssetInfo, // collateral asset is not needed for borrow in mars implementation
    ) -> Result<Uint128, MoneyMarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserDebt {
            user: user.to_string(),
            denom: unwrap_native(borrowed_asset)?,
        };

        let query_response: mars_red_bank_types::red_bank::UserDebtResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.amount)
    }

    fn current_ltv(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        _borrowed_asset: AssetInfo, // LTV is global on Mars and doesn't depend on collateral asset
        _collateral_asset: AssetInfo, // LTV is global on Mars and doesn't depend on borrowed asset
    ) -> Result<Decimal, MoneyMarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserPosition {
            user: user.to_string(),
        };

        let query_response: mars_red_bank_types::red_bank::UserPositionResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        if query_response.total_enabled_collateral.is_zero() {
            return Ok(Decimal::zero());
        }

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
    ) -> Result<Decimal, MoneyMarketError> {
        let market_msg = mars_red_bank_types::red_bank::QueryMsg::UserPosition {
            user: user.to_string(),
        };

        let query_response: mars_red_bank_types::red_bank::UserPositionResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        if query_response.total_enabled_collateral.is_zero() {
            return Ok(Decimal::zero());
        }

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

#[cfg(feature = "full_integration")]
fn unwrap_native(asset: AssetInfo) -> Result<String, MoneyMarketError> {
    match asset {
        cw_asset::AssetInfoBase::Native(denom) => Ok(denom),
        cw_asset::AssetInfoBase::Cw20(_) => Err(MoneyMarketError::ExpectedNative {}),
        _ => todo!(),
    }
}
