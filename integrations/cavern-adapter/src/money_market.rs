use abstract_money_market_standard::Identify;

use crate::{AVAILABLE_CHAINS, CAVERN};
use cosmwasm_std::Addr;
#[derive(Default)]
pub struct Cavern {
    pub oracle_contract: Option<Addr>,
    pub addr_as_sender: Option<Addr>,
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
        feature_objects::AnsHost,
        std::objects::{ans_host::AnsHostError, AssetEntry, ContractEntry},
    },
    cavern_lsd_wrapper_token::state::LSD_CONFIG_KEY,
    cavern_lsd_wrapper_token::trait_def::LSDHub,
    cosmwasm_std::{
        to_json_binary, wasm_execute, CosmosMsg, Decimal, Deps, Env, QuerierWrapper, StdError,
        Uint128,
    },
    cw_asset::{Asset, AssetInfo},
    cw_storage_plus::Item,
    wrapper_implementations::coin::StrideLSDConfig,
};

#[cfg(feature = "full_integration")]
impl MoneyMarketCommand for Cavern {
    fn fetch_data(
        &mut self,
        addr_as_sender: Addr,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> Result<(), MoneyMarketError> {
        let contract_entry = ContractEntry {
            protocol: self.name().to_string(),
            contract: "oracle".to_string(),
        };

        self.oracle_contract = Some(ans_host.query_contract(querier, &contract_entry)?);
        self.addr_as_sender = Some(addr_as_sender);
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
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        // The asset is the collateral. The custody contract tells us which token should actually be deposited

        let custody_config: moneymarket::custody::LSDConfigResponse =
            deps.querier.query_wasm_smart(
                contract_addr.clone(),
                &moneymarket::custody::QueryMsg::Config {},
            )?;

        let msgs = match &asset.info {
            cw_asset::AssetInfoBase::Native(_) => {
                // For native tokens,we :
                // Wrap the tokens into the collateral wrapper
                // Deposit them into the custody contract
                // Lock the collaterals in to be able to borrow with them
                let vault_msg =
                    moneymarket::custody::Cw20HookMsg::DepositCollateral { borrower: None };

                let mint_with_message = wasm_execute(
                    &custody_config.collateral_token,
                    &basset::wrapper::ExecuteMsg::MintWith {
                        recipient: self.addr_as_sender.clone().unwrap().to_string(),
                        lsd_amount: asset.amount,
                    },
                    vec![asset.clone().try_into()?],
                )?;

                let exchange_rate = self.collateral_exchange_rate(deps, contract_addr.clone())?;

                let resulting_wrapped_amount = exchange_rate * asset.amount;

                let wrapped_collateral_asset = Asset::cw20(
                    deps.api.addr_validate(&custody_config.collateral_token)?,
                    resulting_wrapped_amount,
                );

                let lock_msg = wasm_execute(
                    custody_config.overseer_contract.to_string(),
                    &moneymarket::overseer::ExecuteMsg::LockCollateral {
                        collaterals: vec![(
                            custody_config.collateral_token,
                            resulting_wrapped_amount.into(),
                        )],
                    },
                    vec![],
                )?;
                vec![
                    mint_with_message.into(),
                    wrapped_collateral_asset
                        .send_msg(contract_addr, to_json_binary(&vault_msg)?)?,
                    lock_msg.into(),
                ]
            }
            _ => unimplemented!(),
        };

        Ok(msgs)
    }

    fn withdraw_collateral(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneyMarketError> {
        let custody_config: moneymarket::custody::LSDConfigResponse =
            deps.querier.query_wasm_smart(
                contract_addr.clone(),
                &moneymarket::custody::QueryMsg::Config {},
            )?;

        let exchange_rate = self.collateral_exchange_rate(deps, contract_addr.clone())?;

        let resulting_wrapped_amount = exchange_rate * asset.amount;

        let unlock_msg = wasm_execute(
            custody_config.overseer_contract.to_string(),
            &moneymarket::overseer::ExecuteMsg::UnlockCollateral {
                collaterals: vec![(
                    custody_config.collateral_token.clone(),
                    resulting_wrapped_amount.into(),
                )],
            },
            vec![],
        )?;
        let withdraw_msg = moneymarket::custody::ExecuteMsg::WithdrawCollateral {
            amount: Some(resulting_wrapped_amount.into()),
        };

        let burn_msg = basset::wrapper::ExecuteMsg::Burn {
            amount: resulting_wrapped_amount,
        };

        println!(
            "amount: {}, wrapped_amount : {}, exchange_rate, {}",
            asset.amount, resulting_wrapped_amount, exchange_rate
        );

        let msgs = vec![
            unlock_msg.into(),
            wasm_execute(contract_addr, &withdraw_msg, vec![])?.into(),
            wasm_execute(&custody_config.collateral_token, &burn_msg, vec![])?.into(),
        ];

        Ok(msgs)
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

        let msg = wasm_execute(contract_addr, &vault_msg, vec![])?;

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
        _deps: Deps,
        _base: AssetInfo,
        _quote: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        Err(MoneyMarketError::NotImplemented("Price query not implemented for Cavern, because cavern doesn't handle collaterals and doesn't have denoms inscribed normally".to_string()))
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

        let query_response: moneymarket::custody::BorrowerResponse = deps
            .querier
            .query_wasm_smart(&contract_addr, &custody_msg)?;

        let exchange_rate = self.collateral_exchange_rate(deps, contract_addr.clone())?;

        let amount: Uint128 = (query_response.balance - query_response.spendable).try_into()?;
        let resulting_lsd_amount = amount * (Decimal::one() / exchange_rate);

        Ok(resulting_lsd_amount)
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
        if borrow_limit.is_zero() {
            return Ok(Decimal::zero());
        }
        Ok(Decimal::from_ratio(current_borrow, borrow_limit))
    }

    /// This info is located inside the overseer contract (whitelist query) and only inside there
    fn max_ltv(
        &self,
        _deps: Deps,
        _contract_addr: Addr,
        _user: Addr,                // The max LTV doesn't depend on the user in Cavern
        _borrowed_asset: AssetInfo, // The max LTV doesn't depend on the borrowed asset in Cavern
        _collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneyMarketError> {
        Err(MoneyMarketError::NotImplemented("Max ltv query not implemented for Cavern, because cavern doesn't handle collaterals normally".to_string()))
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
        let custody_contract = ContractEntry {
            protocol: self.name().to_string(),
            contract: format!("custody/{}", collateral_asset),
        };

        ans_host.query_contract(querier, &custody_contract)
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

    fn collateral_exchange_rate(
        &self,
        deps: Deps,
        custody_contract: Addr,
    ) -> Result<Decimal, MoneyMarketError> {
        // We need to apply the exchange rate to get the actual underlying asset total
        let custody_config: moneymarket::custody::LSDConfigResponse =
            deps.querier.query_wasm_smart(
                custody_contract.clone(),
                &moneymarket::custody::QueryMsg::Config {},
            )?;
        let lsd_config: StrideLSDConfig = Item::new(LSD_CONFIG_KEY).query(
            &deps.querier,
            deps.api.addr_validate(&custody_config.collateral_token)?,
        )?;

        // mock_env() is not used inside the lsd config query exchange rate function.
        // See: https://github.com/CavernPerson/cavern-lsd-wrapper/blob/8bcdfc0015423f2b4c47c2c3b3fe4cbcb10cf954/packages/wrapper_implementations/src/coin.rs#L56
        // This is a fix because the direct query is not available on the contracts unfortunately.
        // We can feed dummy information inside this mock_env function
        Ok(lsd_config.query_exchange_rate(deps, mock_env())?)
    }
}

/// This is only used as a fix to feed dummy information to the query_exchange_rate function
#[cfg(feature = "full_integration")]
fn mock_env() -> Env {
    use cosmwasm_std::{BlockInfo, ContractInfo};

    Env {
        block: BlockInfo {
            height: Default::default(),
            time: Default::default(),
            chain_id: Default::default(),
        },
        transaction: None,
        contract: ContractInfo {
            address: Addr::unchecked(String::new()),
        },
    }
}
