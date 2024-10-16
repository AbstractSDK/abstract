use crate::MONEY_MARKET_ADAPTER_ID;
use abstract_adapter::sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
};
use abstract_adapter::std::objects::{module::ModuleId, AnsAsset, AssetEntry};
use abstract_money_market_standard::{
    ans_action::MoneyMarketAnsAction,
    msg::{MoneyMarketExecuteMsg, MoneyMarketName, MoneyMarketQueryMsg},
    raw_action::{MoneyMarketRawAction, MoneyMarketRawRequest},
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{Addr, CosmosMsg, Deps};
use cw_asset::{Asset, AssetInfo};

use self::{ans::AnsMoneyMarket, raw::MoneyMarket};

// API for Abstract SDK users
/// Interact with the money_market adapter in your module.
pub trait MoneyMarketInterface:
    AccountIdentification + Dependencies + ModuleIdentification
{
    /// Construct a new money_market interface.
    fn money_market<'a>(&'a self, deps: Deps<'a>, name: MoneyMarketName) -> MoneyMarket<Self> {
        MoneyMarket {
            base: self,
            deps,
            name,
            module_id: MONEY_MARKET_ADAPTER_ID,
        }
    }
    /// Construct a new money_market interface with ANS support.
    fn ans_money_market<'a>(
        &'a self,
        deps: Deps<'a>,
        name: MoneyMarketName,
    ) -> AnsMoneyMarket<Self> {
        AnsMoneyMarket {
            base: self,
            deps,
            name,
            module_id: MONEY_MARKET_ADAPTER_ID,
        }
    }
}

impl<T: AccountIdentification + Dependencies + ModuleIdentification> MoneyMarketInterface for T {}

pub mod raw {
    use cosmwasm_std::{Decimal, Uint128};

    use super::*;

    #[derive(Clone)]
    pub struct MoneyMarket<'a, T: MoneyMarketInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: MoneyMarketName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
    }

    impl<'a, T: MoneyMarketInterface> MoneyMarket<'a, T> {
        /// Set the module id for the MONEY_MARKET
        pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
            Self { module_id, ..self }
        }

        /// Use Raw addresses, ids and denoms for money_market-related operations
        pub fn ans(self) -> AnsMoneyMarket<'a, T> {
            AnsMoneyMarket {
                base: self.base,
                name: self.name,
                module_id: self.module_id,
                deps: self.deps,
            }
        }

        /// returns MONEY_MARKET name
        fn money_market_name(&self) -> MoneyMarketName {
            self.name.clone()
        }

        /// returns the MONEY_MARKET module id
        fn money_market_module_id(&self) -> ModuleId {
            self.module_id
        }

        /// Executes a [MoneyMarketRawAction] in th MONEY_MARKET
        fn execute(&self, action: MoneyMarketRawAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.execute(
                self.money_market_module_id(),
                MoneyMarketExecuteMsg::RawAction {
                    money_market: self.money_market_name(),
                    action,
                },
            )
        }

        /// Deposit assets
        pub fn deposit(
            &self,
            contract_addr: Addr,
            lending_asset: Asset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneyMarketRawRequest::Deposit {
                    lending_asset: lending_asset.into(),
                },
            })
        }

        /// Withdraw liquidity from MONEY_MARKET
        pub fn withdraw(
            &self,
            contract_addr: Addr,
            lent_asset: Asset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneyMarketRawRequest::Withdraw {
                    lent_asset: lent_asset.into(),
                },
            })
        }

        /// Deposit Collateral in MONEY_MARKET
        pub fn provide_collateral(
            &self,
            contract_addr: Addr,
            collateral_asset: Asset,
            borrowable_asset: AssetInfo,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneyMarketRawRequest::ProvideCollateral {
                    collateral_asset: collateral_asset.into(),
                    borrowable_asset: borrowable_asset.into(),
                },
            })
        }

        /// Withdraw collateral liquidity from MONEY_MARKET
        pub fn withdraw_collateral(
            &self,
            contract_addr: Addr,
            collateral_asset: Asset,
            borrowable_asset: AssetInfo,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneyMarketRawRequest::WithdrawCollateral {
                    collateral_asset: collateral_asset.into(),
                    borrowable_asset: borrowable_asset.into(),
                },
            })
        }

        /// Borrow from MoneyMarket
        pub fn borrow(
            &self,
            contract_addr: Addr,
            collateral_asset: AssetInfo,
            borrow_asset: Asset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneyMarketRawRequest::Borrow {
                    collateral_asset: collateral_asset.into(),
                    borrow_asset: borrow_asset.into(),
                },
            })
        }

        /// Repay borrowed assets from MoneyMarket
        pub fn repay(
            &self,
            contract_addr: Addr,
            collateral_asset: AssetInfo,
            borrowed_asset: Asset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneyMarketRawRequest::Repay {
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),
                },
            })
        }
    }

    impl<'a, T: MoneyMarketInterface> MoneyMarket<'a, T> {
        // Queries
        pub fn user_deposit(
            &self,
            user: String,
            asset: AssetInfo,
            contract_addr: String,
        ) -> AbstractSdkResult<Uint128> {
            self.query(MoneyMarketQueryMsg::RawUserDeposit {
                user,
                asset: asset.into(),
                contract_addr,
                money_market: self.money_market_name(),
            })
        }
        pub fn user_collateral(
            &self,
            user: String,
            collateral_asset: AssetInfo,
            borrowed_asset: AssetInfo,
            contract_addr: String,
        ) -> AbstractSdkResult<Uint128> {
            self.query(MoneyMarketQueryMsg::RawUserCollateral {
                user,
                borrowed_asset: borrowed_asset.into(),
                collateral_asset: collateral_asset.into(),
                contract_addr,
                money_market: self.money_market_name(),
            })
        }
        pub fn user_borrow(
            &self,
            user: String,
            collateral_asset: AssetInfo,
            borrowed_asset: AssetInfo,
            contract_addr: String,
        ) -> AbstractSdkResult<Uint128> {
            self.query(MoneyMarketQueryMsg::RawUserBorrow {
                user,
                borrowed_asset: borrowed_asset.into(),
                collateral_asset: collateral_asset.into(),
                contract_addr,
                money_market: self.money_market_name(),
            })
        }
        pub fn current_ltv(
            &self,
            user: String,
            collateral_asset: AssetInfo,
            borrowed_asset: AssetInfo,
            contract_addr: String,
        ) -> AbstractSdkResult<Decimal> {
            self.query(MoneyMarketQueryMsg::RawCurrentLTV {
                user,
                borrowed_asset: borrowed_asset.into(),
                collateral_asset: collateral_asset.into(),
                contract_addr,
                money_market: self.money_market_name(),
            })
        }
        pub fn max_ltv(
            &self,
            user: String,
            collateral_asset: AssetInfo,
            borrowed_asset: AssetInfo,
            contract_addr: String,
        ) -> AbstractSdkResult<Decimal> {
            self.query(MoneyMarketQueryMsg::RawMaxLTV {
                user,
                borrowed_asset: borrowed_asset.into(),
                collateral_asset: collateral_asset.into(),
                contract_addr,
                money_market: self.money_market_name(),
            })
        }
        pub fn price(&self, quote: AssetInfo, base: AssetInfo) -> AbstractSdkResult<Decimal> {
            self.query(MoneyMarketQueryMsg::RawPrice {
                quote: quote.into(),
                base: base.into(),
                money_market: self.money_market_name(),
            })
        }

        /// Do a query in the MONEY_MARKET
        pub fn query<R: DeserializeOwned>(
            &self,
            query_msg: MoneyMarketQueryMsg,
        ) -> AbstractSdkResult<R> {
            let adapters = self.base.adapters(self.deps);
            adapters.query(MONEY_MARKET_ADAPTER_ID, query_msg)
        }
    }
}

pub mod ans {
    use super::*;

    use cosmwasm_std::{Decimal, Uint128};

    #[derive(Clone)]
    pub struct AnsMoneyMarket<'a, T: MoneyMarketInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: MoneyMarketName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
    }

    impl<'a, T: MoneyMarketInterface> AnsMoneyMarket<'a, T> {
        /// Set the module id for the MONEY_MARKET
        pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
            Self { module_id, ..self }
        }

        /// Use Raw addresses, ids and denoms for money_market-related operations
        pub fn raw(self) -> MoneyMarket<'a, T> {
            MoneyMarket {
                base: self.base,
                name: self.name,
                module_id: self.module_id,
                deps: self.deps,
            }
        }

        /// returns MONEY_MARKET name
        fn money_market_name(&self) -> MoneyMarketName {
            self.name.clone()
        }

        /// returns the MONEY_MARKET module id
        fn money_market_module_id(&self) -> ModuleId {
            self.module_id
        }

        /// Executes a [MoneyMarketAction] in th MONEY_MARKET
        fn execute(&self, action: MoneyMarketAnsAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.execute(
                self.money_market_module_id(),
                MoneyMarketExecuteMsg::AnsAction {
                    money_market: self.money_market_name(),
                    action,
                },
            )
        }

        /// Deposit assets
        pub fn deposit(&self, lending_asset: AnsAsset) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketAnsAction::Deposit { lending_asset })
        }

        /// Withdraw liquidity from MONEY_MARKET
        pub fn withdraw(&self, lent_asset: AnsAsset) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketAnsAction::Withdraw { lent_asset })
        }

        /// Deposit Collateral in MONEY_MARKET
        pub fn provide_collateral(
            &self,
            collateral_asset: AnsAsset,
            borrowable_asset: AssetEntry,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketAnsAction::ProvideCollateral {
                collateral_asset,
                borrowable_asset,
            })
        }

        /// Withdraw collateral liquidity from MONEY_MARKET
        pub fn withdraw_collateral(
            &self,
            collateral_asset: AnsAsset,
            borrowable_asset: AssetEntry,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketAnsAction::WithdrawCollateral {
                collateral_asset,
                borrowable_asset,
            })
        }

        /// Borrow from MoneyMarket
        pub fn borrow(
            &self,
            collateral_asset: AssetEntry,
            borrow_asset: AnsAsset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketAnsAction::Borrow {
                collateral_asset,
                borrow_asset,
            })
        }

        /// Repay borrowed assets from MoneyMarket
        pub fn repay(
            &self,
            collateral_asset: AssetEntry,
            borrowed_asset: AnsAsset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(MoneyMarketAnsAction::Repay {
                collateral_asset,
                borrowed_asset,
            })
        }
    }

    impl<'a, T: MoneyMarketInterface> AnsMoneyMarket<'a, T> {
        // Queries
        pub fn user_deposit(&self, user: String, asset: AssetEntry) -> AbstractSdkResult<Uint128> {
            self.query(MoneyMarketQueryMsg::AnsUserDeposit {
                user,
                asset,
                money_market: self.money_market_name(),
            })
        }
        pub fn user_collateral(
            &self,
            user: String,
            collateral_asset: AssetEntry,
            borrowed_asset: AssetEntry,
        ) -> AbstractSdkResult<Uint128> {
            self.query(MoneyMarketQueryMsg::AnsUserCollateral {
                user,
                borrowed_asset,
                collateral_asset,
                money_market: self.money_market_name(),
            })
        }
        pub fn user_borrow(
            &self,
            user: String,
            collateral_asset: AssetEntry,
            borrowed_asset: AssetEntry,
        ) -> AbstractSdkResult<Uint128> {
            self.query(MoneyMarketQueryMsg::AnsUserBorrow {
                user,
                borrowed_asset,
                collateral_asset,
                money_market: self.money_market_name(),
            })
        }
        pub fn current_ltv(
            &self,
            user: String,
            collateral_asset: AssetEntry,
            borrowed_asset: AssetEntry,
        ) -> AbstractSdkResult<Decimal> {
            self.query(MoneyMarketQueryMsg::AnsCurrentLTV {
                user,
                borrowed_asset,
                collateral_asset,
                money_market: self.money_market_name(),
            })
        }
        pub fn max_ltv(
            &self,
            user: String,
            collateral_asset: AssetEntry,
            borrowed_asset: AssetEntry,
        ) -> AbstractSdkResult<Decimal> {
            self.query(MoneyMarketQueryMsg::AnsMaxLTV {
                user,
                borrowed_asset,
                collateral_asset,
                money_market: self.money_market_name(),
            })
        }
        pub fn price(&self, quote: AssetEntry, base: AssetEntry) -> AbstractSdkResult<Decimal> {
            self.query(MoneyMarketQueryMsg::AnsPrice {
                quote,
                base,
                money_market: self.money_market_name(),
            })
        }

        /// Do a query in the MONEY_MARKET
        pub fn query<R: DeserializeOwned>(
            &self,
            query_msg: MoneyMarketQueryMsg,
        ) -> AbstractSdkResult<R> {
            let adapters = self.base.adapters(self.deps);
            adapters.query(MONEY_MARKET_ADAPTER_ID, query_msg)
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::msg::ExecuteMsg;
    use abstract_adapter::abstract_testing::abstract_mock_querier_builder;
    use abstract_adapter::abstract_testing::module::TEST_MODULE_ID;
    use abstract_adapter::abstract_testing::prelude::{
        test_account, AbstractMockAddrs, AbstractMockQuerier, TEST_ACCOUNT_ID,
    };
    use abstract_adapter::sdk::mock_module::MockModule;
    use abstract_adapter::std::adapter::AdapterRequestMsg;
    use cosmwasm_std::{testing::mock_dependencies, wasm_execute};
    use speculoos::prelude::*;

    fn expected_request_with_test_account(
        request: MoneyMarketExecuteMsg,
        account_addr: &Addr,
    ) -> ExecuteMsg {
        AdapterRequestMsg {
            account_address: Some(account_addr.to_string()),
            request,
        }
        .into()
    }

    #[test]
    fn deposit_msg() {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(TEST_MODULE_ID);
        let abstr = AbstractMockAddrs::new(deps.api);

        let money_market_name = "mars".to_string();
        let asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_account(
            MoneyMarketExecuteMsg::AnsAction {
                money_market: money_market_name,
                action: MoneyMarketAnsAction::Deposit {
                    lending_asset: asset.clone(),
                },
            },
            account.addr(),
        );

        let actual = money_market.deposit(asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn withdraw_msg() {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(TEST_MODULE_ID);
        let abstr = AbstractMockAddrs::new(deps.api);

        let money_market_name = "mars".to_string();
        let asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_account(
            MoneyMarketExecuteMsg::AnsAction {
                money_market: money_market_name,
                action: MoneyMarketAnsAction::Withdraw {
                    lent_asset: asset.clone(),
                },
            },
            account.addr(),
        );

        let actual = money_market.withdraw(asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn provide_collateral_msg() {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(TEST_MODULE_ID);
        let abstr = AbstractMockAddrs::new(deps.api);

        let money_market_name = "mars".to_string();
        let borrowable_asset = AssetEntry::new("usdc");
        let collateral_asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_account(
            MoneyMarketExecuteMsg::AnsAction {
                money_market: money_market_name,
                action: MoneyMarketAnsAction::ProvideCollateral {
                    borrowable_asset: borrowable_asset.clone(),
                    collateral_asset: collateral_asset.clone(),
                },
            },
            account.addr(),
        );

        let actual = money_market.provide_collateral(collateral_asset, borrowable_asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn withdraw_collateral_msg() {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(TEST_MODULE_ID);
        let abstr = AbstractMockAddrs::new(deps.api);

        let money_market_name = "mars".to_string();
        let borrowable_asset = AssetEntry::new("usdc");
        let collateral_asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_account(
            MoneyMarketExecuteMsg::AnsAction {
                money_market: money_market_name,
                action: MoneyMarketAnsAction::WithdrawCollateral {
                    borrowable_asset: borrowable_asset.clone(),
                    collateral_asset: collateral_asset.clone(),
                },
            },
            account.addr(),
        );

        let actual = money_market.withdraw_collateral(collateral_asset, borrowable_asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn borrow_msg() {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(TEST_MODULE_ID);
        let abstr = AbstractMockAddrs::new(deps.api);

        let money_market_name = "mars".to_string();
        let collateral_asset = AssetEntry::new("juno");
        let borrow_asset = AnsAsset::new("usdc", 1000u128);

        let expected = expected_request_with_test_account(
            MoneyMarketExecuteMsg::AnsAction {
                money_market: money_market_name,
                action: MoneyMarketAnsAction::Borrow {
                    borrow_asset: borrow_asset.clone(),
                    collateral_asset: collateral_asset.clone(),
                },
            },
            account.addr(),
        );

        let actual = money_market.borrow(collateral_asset, borrow_asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn repay_msg() {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(TEST_MODULE_ID);
        let abstr = AbstractMockAddrs::new(deps.api);

        let money_market_name = "mars".to_string();
        let collateral_asset = AssetEntry::new("juno");
        let borrowed_asset = AnsAsset::new("usdc", 1000u128);

        let expected = expected_request_with_test_account(
            MoneyMarketExecuteMsg::AnsAction {
                money_market: money_market_name,
                action: MoneyMarketAnsAction::Repay {
                    borrowed_asset: borrowed_asset.clone(),
                    collateral_asset: collateral_asset.clone(),
                },
            },
            account.addr(),
        );

        let actual = money_market.repay(collateral_asset, borrowed_asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    mod raw {
        use super::*;

        pub const TEST_CONTRACT_ADDR: &str = "test-mm-addr";

        #[test]
        fn deposit_msg() {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();
            let stub = MockModule::new(deps.api, account.clone());
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(TEST_MODULE_ID);
            let abstr = AbstractMockAddrs::new(deps.api);

            let money_market_name = "mars".to_string();
            let asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_account(
                MoneyMarketExecuteMsg::RawAction {
                    money_market: money_market_name,
                    action: MoneyMarketRawAction {
                        contract_addr: deps.api.addr_make(TEST_CONTRACT_ADDR).to_string(),
                        request: MoneyMarketRawRequest::Deposit {
                            lending_asset: asset.clone().into(),
                        },
                    },
                },
                account.addr(),
            );

            let actual = money_market.deposit(deps.api.addr_make(TEST_CONTRACT_ADDR), asset);

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn withdraw_msg() {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();
            let stub = MockModule::new(deps.api, account.clone());
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(TEST_MODULE_ID);
            let abstr = AbstractMockAddrs::new(deps.api);

            let money_market_name = "mars".to_string();
            let asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_account(
                MoneyMarketExecuteMsg::RawAction {
                    money_market: money_market_name,
                    action: MoneyMarketRawAction {
                        contract_addr: deps.api.addr_make(TEST_CONTRACT_ADDR).to_string(),
                        request: MoneyMarketRawRequest::Withdraw {
                            lent_asset: asset.clone().into(),
                        },
                    },
                },
                account.addr(),
            );

            let actual = money_market.withdraw(deps.api.addr_make(TEST_CONTRACT_ADDR), asset);

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn provide_collateral_msg() {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();
            let stub = MockModule::new(deps.api, account.clone());
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(TEST_MODULE_ID);
            let abstr = AbstractMockAddrs::new(deps.api);

            let money_market_name = "mars".to_string();
            let borrowable_asset = AssetInfo::native("usdc");
            let collateral_asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_account(
                MoneyMarketExecuteMsg::RawAction {
                    money_market: money_market_name,
                    action: MoneyMarketRawAction {
                        contract_addr: deps.api.addr_make(TEST_CONTRACT_ADDR).to_string(),
                        request: MoneyMarketRawRequest::ProvideCollateral {
                            borrowable_asset: borrowable_asset.clone().into(),
                            collateral_asset: collateral_asset.clone().into(),
                        },
                    },
                },
                account.addr(),
            );

            let actual = money_market.provide_collateral(
                deps.api.addr_make(TEST_CONTRACT_ADDR),
                collateral_asset,
                borrowable_asset,
            );

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn withdraw_collateral_msg() {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();
            let stub = MockModule::new(deps.api, account.clone());
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(TEST_MODULE_ID);
            let abstr = AbstractMockAddrs::new(deps.api);

            let money_market_name = "mars".to_string();
            let borrowable_asset = AssetInfo::native("usdc");
            let collateral_asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_account(
                MoneyMarketExecuteMsg::RawAction {
                    money_market: money_market_name,
                    action: MoneyMarketRawAction {
                        contract_addr: deps.api.addr_make(TEST_CONTRACT_ADDR).to_string(),
                        request: MoneyMarketRawRequest::WithdrawCollateral {
                            borrowable_asset: borrowable_asset.clone().into(),
                            collateral_asset: collateral_asset.clone().into(),
                        },
                    },
                },
                account.addr(),
            );

            let actual = money_market.withdraw_collateral(
                deps.api.addr_make(TEST_CONTRACT_ADDR),
                collateral_asset,
                borrowable_asset,
            );

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn borrow_msg() {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();
            let stub = MockModule::new(deps.api, account.clone());
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(TEST_MODULE_ID);
            let abstr = AbstractMockAddrs::new(deps.api);

            let money_market_name = "mars".to_string();
            let collateral_asset = AssetInfo::native("juno");
            let borrow_asset = Asset::native("usdc", 1000u128);

            let expected = expected_request_with_test_account(
                MoneyMarketExecuteMsg::RawAction {
                    money_market: money_market_name,
                    action: MoneyMarketRawAction {
                        contract_addr: deps.api.addr_make(TEST_CONTRACT_ADDR).to_string(),
                        request: MoneyMarketRawRequest::Borrow {
                            borrow_asset: borrow_asset.clone().into(),
                            collateral_asset: collateral_asset.clone().into(),
                        },
                    },
                },
                account.addr(),
            );

            let actual = money_market.borrow(
                deps.api.addr_make(TEST_CONTRACT_ADDR),
                collateral_asset,
                borrow_asset,
            );

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn repay_msg() {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();
            let stub = MockModule::new(deps.api, account.clone());
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(TEST_MODULE_ID);
            let abstr = AbstractMockAddrs::new(deps.api);

            let money_market_name = "mars".to_string();
            let collateral_asset = AssetInfo::native("juno");
            let borrowed_asset = Asset::native("usdc", 1000u128);

            let expected = expected_request_with_test_account(
                MoneyMarketExecuteMsg::RawAction {
                    money_market: money_market_name,
                    action: MoneyMarketRawAction {
                        contract_addr: deps.api.addr_make(TEST_CONTRACT_ADDR).to_string(),
                        request: MoneyMarketRawRequest::Repay {
                            borrowed_asset: borrowed_asset.clone().into(),
                            collateral_asset: collateral_asset.clone().into(),
                        },
                    },
                },
                account.addr(),
            );

            let actual = money_market.repay(
                deps.api.addr_make(TEST_CONTRACT_ADDR),
                collateral_asset,
                borrowed_asset,
            );

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

            assert_that!(actual).is_equal_to(expected);
        }
    }
}
