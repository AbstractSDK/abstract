use crate::{AVAILABLE_CHAINS, KUJIRA};
use abstract_moneymarket_standard::Identify;

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
    abstract_moneymarket_standard::{MoneymarketCommand, MoneymarketError},
    abstract_sdk::{
        core::objects::{ans_host::AnsHostError, AssetEntry, ContractEntry},
        feature_objects::AnsHost,
    },
    cosmwasm_std::{coins, wasm_execute, Addr, CosmosMsg, Decimal, Deps, QuerierWrapper, Uint128},
    cw_asset::{Asset, AssetInfo},
    kujira::{
        ghost::{
            market::{self},
            receipt_vault,
        },
        KujiraQuerier,
    },
};

#[cfg(feature = "full_integration")]
impl MoneymarketCommand for Kujira {
    fn deposit(
        &self,
        _deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let vault_msg =
            receipt_vault::ExecuteMsg::Deposit(receipt_vault::DepositMsg { callback: None });

        let msg = wasm_execute(contract_addr, &vault_msg, vec![asset.try_into()?])?;

        Ok(vec![msg.into()])
    }

    fn withdraw(
        &self,
        deps: Deps,
        contract_addr: Addr,
        asset: Asset,
    ) -> Result<Vec<CosmosMsg>, MoneymarketError> {
        let config: receipt_vault::query::ConfigResponse = deps
            .querier
            .query_wasm_smart(&contract_addr, &receipt_vault::query::QueryMsg::Config {})?;
        let status: receipt_vault::query::StatusResponse = deps
            .querier
            .query_wasm_smart(&contract_addr, &receipt_vault::query::QueryMsg::Status {})?;

        let vault_msg =
            receipt_vault::ExecuteMsg::Withdraw(receipt_vault::WithdrawMsg { callback: None });

        let msg = wasm_execute(
            contract_addr,
            &vault_msg,
            coins(
                ((Decimal::from_ratio(asset.amount, 1u128) / status.deposit_redemption_ratio)
                    * Uint128::one())
                .u128(),
                config.receipt_denom,
            ),
        )?;

        Ok(vec![msg.into()])
    }

    fn provide_collateral(
        &self,
        _deps: Deps,
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
        _deps: Deps,
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
        _deps: Deps,
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
        _deps: Deps,
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
        vault_addr: Addr,
        user: Addr,
        _asset: AssetInfo, // vault_addr is already lending asset specific
    ) -> Result<Uint128, MoneymarketError> {
        // We get the lending receipt denom
        let config: receipt_vault::query::ConfigResponse = deps
            .querier
            .query_wasm_smart(vault_addr, &receipt_vault::query::QueryMsg::Config {})?;

        // We get the balance of that token denom
        let balance = deps.querier.query_balance(user, config.receipt_denom)?;

        Ok(balance.amount)
    }

    fn user_collateral(
        &self,
        deps: Deps,
        market_addr: Addr,
        user: Addr,
        _borrowed_asset: AssetInfo, // market_addr is already borrowed asset specific
        _collateral_asset: AssetInfo, // market_addr is already collateral asset specific
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = market::QueryMsg::Position { holder: user };

        let query_response: market::PositionResponse =
            deps.querier.query_wasm_smart(market_addr, &market_msg)?;

        Ok(query_response.collateral_amount)
    }

    fn user_borrow(
        &self,
        deps: Deps,
        contract_addr: Addr,
        user: Addr,
        _borrowed_asset: AssetInfo, // market_addr is already borrowed asset specific
        _collateral_asset: AssetInfo, // market_addr is already collateral asset specific
    ) -> Result<Uint128, MoneymarketError> {
        let market_msg = market::QueryMsg::Position { holder: user };

        let query_response: market::PositionResponse =
            deps.querier.query_wasm_smart(contract_addr, &market_msg)?;

        Ok(query_response.debt_shares)
    }

    fn current_ltv(
        &self,
        deps: Deps,
        market_addr: Addr,
        user: Addr,
        borrowed_asset: AssetInfo,
        collateral_asset: AssetInfo,
    ) -> Result<Decimal, MoneymarketError> {
        // We get the borrowed_value / collateral value
        let collateral = self.user_collateral(
            deps,
            market_addr.clone(),
            user.clone(),
            collateral_asset.clone(),
            borrowed_asset.clone(),
        )?;
        let borrow = self.user_borrow(
            deps,
            market_addr,
            user,
            collateral_asset.clone(),
            borrowed_asset.clone(),
        )?;

        // This represents how much 1 unit of the collateral_asset is worth in terms of the borrowed_asset
        let collateral_price = self.price(deps, collateral_asset, borrowed_asset)?;

        let collateral_value = Decimal::from_ratio(collateral, 1u128) * collateral_price;

        Ok(Decimal::from_ratio(borrow, 1u128) / collateral_value)
    }

    fn max_ltv(
        &self,
        deps: Deps,
        market_addr: Addr,
        _user: Addr, // This info is not user specific in this money market
        _collateral_asset: AssetInfo, // market_addr is already collateral asset specific
    ) -> Result<Decimal, MoneymarketError> {
        let market_msg = market::QueryMsg::Config {};

        let query_response: market::ConfigResponse =
            deps.querier.query_wasm_smart(market_addr, &market_msg)?;

        Ok(query_response.max_ltv)
    }

    fn lending_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.vault_address(querier, ans_host, lending_asset)
    }

    fn collateral_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.market_address(querier, ans_host, lending_asset, collateral_asset)
    }

    fn borrow_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.market_address(querier, ans_host, lending_asset, collateral_asset)
    }

    fn max_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.market_address(querier, ans_host, lending_asset, collateral_asset)
    }

    fn current_ltv_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        self.market_address(querier, ans_host, lending_asset, collateral_asset)
    }
}

#[cfg(feature = "full_integration")]
impl Kujira {
    fn vault_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        let vault_contract = ContractEntry {
            protocol: self.name().to_string(),
            contract: format!("vault/{}", lending_asset),
        };

        ans_host.query_contract(querier, &vault_contract)
    }
    fn market_address(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
        lending_asset: AssetEntry,
        collateral_asset: AssetEntry,
    ) -> Result<Addr, AnsHostError> {
        let market_contract = ContractEntry {
            protocol: self.name().to_string(),
            contract: format!("market/{}/{}", lending_asset, collateral_asset),
        };

        ans_host.query_contract(querier, &market_contract)
    }
}
