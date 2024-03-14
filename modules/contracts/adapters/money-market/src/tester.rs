use std::str::FromStr;

use crate::{
    interface::MoneyMarketAdapter, msg::MoneyMarketInstantiateMsg, MONEYMARKET_ADAPTER_ID,
};
use abstract_client::{AbstractClient, Environment};
use abstract_core::{
    adapter,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        AnsAsset, AssetEntry, UncheckedContractEntry,
    },
};
use abstract_interface::{AdapterDeployer, DeployStrategy, ExecuteMsgFns, VCExecFns};
use abstract_money_market_standard::{
    ans_action::MoneyMarketAnsAction, msg::MoneyMarketExecuteMsg, query::MoneyMarketAnsQuery,
};
use cosmwasm_schema::serde::{de::DeserializeOwned, Serialize};
use cosmwasm_std::{coins, Decimal, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{anyhow, environment::MutCwEnv, prelude::*};

pub trait MockMoneyMarket {
    /// Name of the moneymarket
    fn name(&self) -> String;

    /// lending asset
    fn lending_asset(&self) -> (String, cw_asset::AssetInfoUnchecked);

    /// collateral asset
    fn collateral_asset(&self) -> (String, cw_asset::AssetInfoUnchecked);

    /// Specific moneymarket setup
    /// Should return objects that will be registered inside abstract ANS
    fn setup(&self) -> Vec<(UncheckedContractEntry, String)>;
}
pub struct MoneyMarketTester<Chain: MutCwEnv, Moneymarket: MockMoneyMarket> {
    pub abstr_deployment: AbstractClient<Chain>,
    pub moneymarket_adapter: MoneyMarketAdapter<Chain>,
    pub moneymarket: Moneymarket,
}
impl<Chain: MutCwEnv, Moneymarket: MockMoneyMarket> MoneyMarketTester<Chain, Moneymarket> {
    pub fn new(
        abstr_deployment: AbstractClient<Chain>,
        moneymarket: Moneymarket,
    ) -> anyhow::Result<Self> {
        // Re-register moneymarket, to make sure it's latest
        let _ = abstr_deployment
            .version_control()
            .remove_module(ModuleInfo::from_id(
                MONEYMARKET_ADAPTER_ID,
                ModuleVersion::Version(crate::contract::CONTRACT_VERSION.to_owned()),
            )?);
        let moneymarket_adapter =
            MoneyMarketAdapter::new(MONEYMARKET_ADAPTER_ID, abstr_deployment.environment());
        moneymarket_adapter.deploy(
            crate::contract::CONTRACT_VERSION.parse()?,
            MoneyMarketInstantiateMsg {
                recipient_account: 0,
                fee: Decimal::permille(3),
            },
            DeployStrategy::Force,
        )?;

        // Registering assets
        abstr_deployment.name_service().update_asset_addresses(
            vec![moneymarket.lending_asset(), moneymarket.collateral_asset()],
            vec![],
        )?;

        let new_contract_entries = moneymarket.setup();
        abstr_deployment
            .name_service()
            .update_contract_addresses(new_contract_entries, vec![])?;

        Ok(Self {
            abstr_deployment,
            moneymarket_adapter,
            moneymarket,
        })
    }

    pub fn test_deposit(&self) -> anyhow::Result<()> {
        let (ans_lending_asset, asset_info_lending) = self.moneymarket.lending_asset();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<MoneyMarketAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        let deposit_value = 1_000_000_000u128;

        self.add_proxy_balance(&proxy_addr, &asset_info_lending, deposit_value)?;

        // Verify nothing was deposited using the moneymarket query
        let user_deposit: Uint128 = self.query(MoneyMarketAnsQuery::UserDeposit {
            user: new_account.proxy()?.to_string(),
            asset: AssetEntry::new(&ans_lending_asset),
        })?;

        assert_eq!(user_deposit.u128(), 0);

        // swap 1_000_000_000 asset_a to asset_b
        self.execute(
            &proxy_addr,
            MoneyMarketAnsAction::Deposit {
                lending_asset: AnsAsset::new(AssetEntry::new(&ans_lending_asset), deposit_value),
            },
        )?;

        // Assert balances
        let balance_lending = self.query_proxy_balance(&proxy_addr, &asset_info_lending)?;
        assert!(balance_lending.is_zero());

        // Verify the deposit using the moneymarket query
        let user_deposit: Uint128 = self.query(MoneyMarketAnsQuery::UserDeposit {
            user: new_account.proxy()?.to_string(),
            asset: AssetEntry::new(&ans_lending_asset),
        })?;

        assert!(user_deposit > Uint128::from(deposit_value) * Decimal::from_str("0.95")?);
        assert_eq!(
            self.abstr_deployment
                .environment()
                .bank_querier()
                .balance(proxy_addr, None)
                .unwrap()
                .len(),
            0
        );

        Ok(())
    }

    pub fn test_withdraw(&self) -> anyhow::Result<()> {
        let (ans_lending_asset, asset_info_lending) = self.moneymarket.lending_asset();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<MoneyMarketAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        let deposit_value = 1_000_000_000u128;

        self.add_proxy_balance(&proxy_addr, &asset_info_lending, deposit_value)?;

        // swap 1_000_000_000 asset_a to asset_b
        self.execute(
            &proxy_addr,
            MoneyMarketAnsAction::Deposit {
                lending_asset: AnsAsset::new(AssetEntry::new(&ans_lending_asset), deposit_value),
            },
        )?;
        let current_balance = self.query_proxy_balance(&proxy_addr, &asset_info_lending)?;
        assert_eq!(current_balance, Uint128::zero());

        // Verify the deposit using the moneymarket query
        let user_deposit_value: Uint128 = self.query(MoneyMarketAnsQuery::UserDeposit {
            user: new_account.proxy()?.to_string(),
            asset: AssetEntry::new(&ans_lending_asset),
        })?;
        self.execute(
            &proxy_addr,
            MoneyMarketAnsAction::Withdraw {
                lent_asset: AnsAsset::new(
                    AssetEntry::new(&ans_lending_asset),
                    user_deposit_value.clone(),
                ),
            },
        )?;

        let current_balance = self.query_proxy_balance(&proxy_addr, &asset_info_lending)?;
        assert_eq!(current_balance, user_deposit_value);

        Ok(())
    }

    fn query<T: Serialize + std::fmt::Debug + DeserializeOwned>(
        &self,
        query: MoneyMarketAnsQuery,
    ) -> anyhow::Result<T> {
        Ok(self
            .moneymarket_adapter
            .query(&crate::msg::QueryMsg::Module(
                crate::msg::MoneyMarketQueryMsg::MoneyMarketAnsQuery {
                    query,
                    money_market: self.moneymarket.name(),
                },
            ))?)
    }

    fn execute(
        &self,
        proxy: &Addr,
        action: MoneyMarketAnsAction,
    ) -> anyhow::Result<<Chain as TxHandler>::Response> {
        Ok(self.moneymarket_adapter.execute(
            &crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: Some(proxy.to_string()),
                request: MoneyMarketExecuteMsg::AnsAction {
                    money_market: self.moneymarket.name(),
                    action,
                },
            }),
            None,
        )?)
    }

    fn add_proxy_balance(
        &self,
        proxy_addr: &Addr,
        asset: &AssetInfoUnchecked,
        amount: u128,
    ) -> anyhow::Result<()> {
        let mut chain = self.abstr_deployment.environment();

        match asset {
            cw_asset::AssetInfoBase::Native(denom) => {
                chain.add_balance(proxy_addr, coins(amount, denom))?;
            }
            cw_asset::AssetInfoBase::Cw20(addr) => {
                chain.execute(
                    &cw20::Cw20ExecuteMsg::Mint {
                        recipient: proxy_addr.to_string(),
                        amount: amount.into(),
                    },
                    &[],
                    &Addr::unchecked(addr),
                )?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn query_proxy_balance(
        &self,
        proxy_addr: &Addr,
        asset: &AssetInfoUnchecked,
    ) -> anyhow::Result<Uint128> {
        let chain = self.abstr_deployment.environment();

        let balance = match asset {
            cw_asset::AssetInfoBase::Native(denom) => {
                chain
                    .bank_querier()
                    .balance(proxy_addr, Some(denom.to_owned()))
                    .unwrap()
                    .pop()
                    .unwrap()
                    .amount
            }
            cw_asset::AssetInfoBase::Cw20(addr) => {
                let balance: cw20::BalanceResponse = chain
                    .query(
                        &cw20::Cw20QueryMsg::Balance {
                            address: proxy_addr.to_string(),
                        },
                        &Addr::unchecked(addr),
                    )
                    .unwrap();
                balance.balance
            }
            _ => unreachable!(),
        };

        Ok(balance)
    }
}
