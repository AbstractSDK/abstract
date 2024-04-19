use std::str::FromStr;

use crate::{
    interface::MoneyMarketAdapter, msg::MoneyMarketInstantiateMsg, MONEY_MARKET_ADAPTER_ID,
};
use abstract_client::{AbstractClient, Account, Environment};
use abstract_interface::{AdapterDeployer, DeployStrategy, ExecuteMsgFns, VCExecFns};
use abstract_money_market_standard::{
    ans_action::MoneyMarketAnsAction,
    msg::{MoneyMarketExecuteMsg, MoneyMarketQueryMsg},
};
use abstract_std::{
    adapter,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        AnsAsset, AssetEntry, UncheckedContractEntry,
    },
};
use cosmwasm_schema::serde::{de::DeserializeOwned, Serialize};
use cosmwasm_std::{coins, Decimal, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{anyhow, environment::MutCwEnv, prelude::*};

pub const BORROW_VALUE: u128 = 1_000_000u128;
pub const DEPOSIT_VALUE: u128 = 1_000_000_000u128;

pub const PER_MILLE_FEE: u64 = 3;
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
                MONEY_MARKET_ADAPTER_ID,
                ModuleVersion::Version(crate::contract::CONTRACT_VERSION.to_owned()),
            )?);
        let moneymarket_adapter =
            MoneyMarketAdapter::new(MONEY_MARKET_ADAPTER_ID, abstr_deployment.environment());
        moneymarket_adapter.deploy(
            crate::contract::CONTRACT_VERSION.parse()?,
            MoneyMarketInstantiateMsg {
                recipient_account: 0,
                fee: Decimal::permille(PER_MILLE_FEE),
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

    pub fn test_deposit(&self) -> anyhow::Result<Account<Chain>> {
        self.deposit(DEPOSIT_VALUE)
    }

    pub fn deposit(&self, amount: u128) -> anyhow::Result<Account<Chain>> {
        let (ans_lending_asset, asset_info_lending) = self.moneymarket.lending_asset();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<MoneyMarketAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        self.add_proxy_balance(&proxy_addr, &asset_info_lending, amount)?;

        // Verify nothing was deposited using the moneymarket query
        let user_deposit: Uint128 = self.query(MoneyMarketQueryMsg::AnsUserDeposit {
            user: new_account.proxy()?.to_string(),
            asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;
        assert_eq!(user_deposit.u128(), 0);

        // swap 1_000_000_000 asset_a to asset_b
        self.execute(
            &proxy_addr,
            MoneyMarketAnsAction::Deposit {
                lending_asset: AnsAsset::new(AssetEntry::new(&ans_lending_asset), amount),
            },
        )?;

        // Assert balances
        let balance_lending = self.query_proxy_balance(&proxy_addr, &asset_info_lending)?;
        assert!(balance_lending.is_zero());

        // Verify the deposit using the moneymarket query
        let user_deposit: Uint128 = self.query(MoneyMarketQueryMsg::AnsUserDeposit {
            user: new_account.proxy()?.to_string(),
            asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;

        assert!(user_deposit > Uint128::from(amount) * Decimal::from_str("0.95")?);
        assert_eq!(
            self.abstr_deployment
                .environment()
                .bank_querier()
                .balance(proxy_addr, None)
                .unwrap()
                .len(),
            0
        );

        Ok(new_account)
    }

    pub fn test_withdraw(&self) -> anyhow::Result<Account<Chain>> {
        let (ans_lending_asset, asset_info_lending) = self.moneymarket.lending_asset();

        let account = self.test_deposit()?;

        // Verify the deposit using the moneymarket query
        let user_deposit_value: Uint128 = self.query(MoneyMarketQueryMsg::AnsUserDeposit {
            user: account.proxy()?.to_string(),
            asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;
        assert!(user_deposit_value > Uint128::from(DEPOSIT_VALUE) * Decimal::percent(99));
        let withdraw_fee = user_deposit_value * Decimal::permille(PER_MILLE_FEE);
        self.execute(
            &account.proxy()?,
            MoneyMarketAnsAction::Withdraw {
                lent_asset: AnsAsset::new(AssetEntry::new(&ans_lending_asset), user_deposit_value),
            },
        )?;

        let current_balance = self.query_proxy_balance(&account.proxy()?, &asset_info_lending)?;
        assert!(current_balance + withdraw_fee > user_deposit_value * Decimal::percent(99));

        Ok(account)
    }

    pub fn test_provide_collateral(&self) -> anyhow::Result<Account<Chain>> {
        let (ans_collateral_asset, asset_info_collateral) = self.moneymarket.collateral_asset();
        let (ans_lending_asset, _asset_info_lending) = self.moneymarket.lending_asset();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<MoneyMarketAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        self.add_proxy_balance(&proxy_addr, &asset_info_collateral, DEPOSIT_VALUE)?;

        // Verify nothing was deposited using the moneymarket query
        let user_collateral: Uint128 = self.query(MoneyMarketQueryMsg::AnsUserCollateral {
            user: new_account.proxy()?.to_string(),
            collateral_asset: AssetEntry::new(&ans_collateral_asset),
            borrowed_asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;
        assert_eq!(user_collateral.u128(), 0);

        self.execute(
            &proxy_addr,
            MoneyMarketAnsAction::ProvideCollateral {
                borrowable_asset: AssetEntry::new(&ans_lending_asset),
                collateral_asset: AnsAsset::new(&ans_collateral_asset, DEPOSIT_VALUE),
            },
        )?;

        // Assert balances
        let balance_collateral = self.query_proxy_balance(&proxy_addr, &asset_info_collateral)?;
        assert!(balance_collateral.is_zero());

        // Verify the deposit using the moneymarket query
        let user_collateral: Uint128 = self.query(MoneyMarketQueryMsg::AnsUserCollateral {
            user: new_account.proxy()?.to_string(),
            collateral_asset: AssetEntry::new(&ans_collateral_asset),
            borrowed_asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;

        assert!(user_collateral > Uint128::from(DEPOSIT_VALUE) * Decimal::from_str("0.95")?);
        assert_eq!(
            self.abstr_deployment
                .environment()
                .bank_querier()
                .balance(proxy_addr, None)
                .unwrap()
                .len(),
            0
        );

        Ok(new_account)
    }

    pub fn test_withdraw_collateral(&self) -> anyhow::Result<Account<Chain>> {
        let (ans_collateral_asset, asset_info_collateral) = self.moneymarket.collateral_asset();
        let (ans_lending_asset, _asset_info_lending) = self.moneymarket.lending_asset();

        let account = self.test_provide_collateral()?;

        // Verify the deposit using the moneymarket query
        let user_collateral: Uint128 = self.query(MoneyMarketQueryMsg::AnsUserCollateral {
            user: account.proxy()?.to_string(),
            collateral_asset: AssetEntry::new(&ans_collateral_asset),
            borrowed_asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;
        assert!(user_collateral > Uint128::from(DEPOSIT_VALUE) * Decimal::from_str("0.95")?);

        self.execute(
            &account.proxy()?,
            MoneyMarketAnsAction::WithdrawCollateral {
                borrowable_asset: AssetEntry::new(&ans_lending_asset),
                collateral_asset: AnsAsset::new(&ans_collateral_asset, user_collateral),
            },
        )?;

        let current_balance =
            self.query_proxy_balance(&account.proxy()?, &asset_info_collateral)?;
        assert!(current_balance > user_collateral * Decimal::percent(99));

        Ok(account)
    }

    pub fn test_borrow(&self) -> anyhow::Result<Account<Chain>> {
        let (ans_collateral_asset, _asset_info_collateral) = self.moneymarket.collateral_asset();
        let (ans_lending_asset, _asset_info_lending) = self.moneymarket.lending_asset();

        let account: Account<Chain> = self.test_provide_collateral()?;
        let proxy_addr = account.proxy()?;

        self.execute(
            &proxy_addr,
            MoneyMarketAnsAction::Borrow {
                borrow_asset: AnsAsset::new(&ans_lending_asset, BORROW_VALUE),
                collateral_asset: AssetEntry::new(&ans_collateral_asset),
            },
        )?;

        let user_borrow: Uint128 = self.query(MoneyMarketQueryMsg::AnsUserBorrow {
            user: account.proxy()?.to_string(),
            collateral_asset: AssetEntry::new(&ans_collateral_asset),
            borrowed_asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;

        assert!(user_borrow > Uint128::from(BORROW_VALUE) * Decimal::percent(99));

        Ok(account)
    }

    pub fn test_repay(&self) -> anyhow::Result<Account<Chain>> {
        let (ans_collateral_asset, _asset_info_collateral) = self.moneymarket.collateral_asset();
        let (ans_lending_asset, _asset_info_lending) = self.moneymarket.lending_asset();
        let account: Account<Chain> = self.test_borrow()?;
        let proxy_addr = account.proxy()?;

        // Now we repay
        self.execute(
            &proxy_addr,
            MoneyMarketAnsAction::Repay {
                borrowed_asset: AnsAsset::new(&ans_lending_asset, BORROW_VALUE),
                collateral_asset: AssetEntry::new(&ans_collateral_asset),
            },
        )?;
        let user_borrow: Uint128 = self.query(MoneyMarketQueryMsg::AnsUserBorrow {
            user: proxy_addr.to_string(),
            collateral_asset: AssetEntry::new(&ans_collateral_asset),
            borrowed_asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;

        assert_eq!(user_borrow.u128(), 0);

        Ok(account)
    }

    pub fn test_price(&self) -> anyhow::Result<()> {
        let (ans_collateral_asset, _asset_info_collateral) = self.moneymarket.collateral_asset();
        let (ans_lending_asset, _asset_info_lending) = self.moneymarket.lending_asset();
        let _price: Decimal = self.query(MoneyMarketQueryMsg::AnsPrice {
            quote: AssetEntry::new(&ans_collateral_asset),
            base: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;

        Ok(())
    }

    pub fn test_user_ltv(&self) -> anyhow::Result<()> {
        let (ans_collateral_asset, _asset_info_collateral) = self.moneymarket.collateral_asset();
        let (ans_lending_asset, _asset_info_lending) = self.moneymarket.lending_asset();

        let account = self.test_borrow()?;

        let ltv: Decimal = self.query(MoneyMarketQueryMsg::AnsCurrentLTV {
            user: account.proxy()?.to_string(),
            collateral_asset: AssetEntry::new(&ans_collateral_asset),
            borrowed_asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;
        assert!(ltv > Decimal::zero());

        Ok(())
    }

    pub fn test_max_ltv(&self) -> anyhow::Result<()> {
        let (ans_collateral_asset, _asset_info_collateral) = self.moneymarket.collateral_asset();
        let (ans_lending_asset, _asset_info_lending) = self.moneymarket.lending_asset();

        let account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<MoneyMarketAdapter<Chain>>()?
            .build()?;

        let _max_ltv: Decimal = self.query(MoneyMarketQueryMsg::AnsMaxLTV {
            user: account.proxy()?.to_string(),
            collateral_asset: AssetEntry::new(&ans_collateral_asset),
            borrowed_asset: AssetEntry::new(&ans_lending_asset),
            money_market: self.moneymarket.name(),
        })?;

        Ok(())
    }

    fn query<T: Serialize + std::fmt::Debug + DeserializeOwned>(
        &self,
        query: MoneyMarketQueryMsg,
    ) -> anyhow::Result<T> {
        Ok(self
            .moneymarket_adapter
            .query(&crate::msg::QueryMsg::Module(query))?)
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
