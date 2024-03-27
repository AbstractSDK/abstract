use crate::MONEY_MARKET_ADAPTER_ID;
use abstract_core::objects::{module::ModuleId, AnsAsset, AssetEntry};
use abstract_money_market_standard::{
    ans_action::MoneyMarketAnsAction,
    msg::{MoneyMarketExecuteMsg, MoneyMarketName, MoneyMarketQueryMsg},
    raw_action::{MoneyMarketRawAction, MoneyMarketRawRequest},
};
use abstract_sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
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
        fn request(&self, action: MoneyMarketRawAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.request(
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
            self.request(MoneyMarketRawAction {
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
            self.request(MoneyMarketRawAction {
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
            self.request(MoneyMarketRawAction {
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
            self.request(MoneyMarketRawAction {
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
            self.request(MoneyMarketRawAction {
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
            self.request(MoneyMarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneyMarketRawRequest::Repay {
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),
                },
            })
        }
    }

    impl<'a, T: MoneyMarketInterface> MoneyMarket<'a, T> {
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
    use cosmwasm_schema::serde::de::DeserializeOwned;

    use self::raw::MoneyMarket;

    use super::*;

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
        fn request(&self, action: MoneyMarketAnsAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.request(
                self.money_market_module_id(),
                MoneyMarketExecuteMsg::AnsAction {
                    money_market: self.money_market_name(),
                    action,
                },
            )
        }

        /// Deposit assets
        pub fn deposit(&self, lending_asset: AnsAsset) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneyMarketAnsAction::Deposit { lending_asset })
        }

        /// Withdraw liquidity from MONEY_MARKET
        pub fn withdraw(&self, lent_asset: AnsAsset) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneyMarketAnsAction::Withdraw { lent_asset })
        }

        /// Deposit Collateral in MONEY_MARKET
        pub fn provide_collateral(
            &self,
            collateral_asset: AnsAsset,
            borrowable_asset: AssetEntry,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneyMarketAnsAction::ProvideCollateral {
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
            self.request(MoneyMarketAnsAction::WithdrawCollateral {
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
            self.request(MoneyMarketAnsAction::Borrow {
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
            self.request(MoneyMarketAnsAction::Repay {
                collateral_asset,
                borrowed_asset,
            })
        }
    }

    impl<'a, T: MoneyMarketInterface> AnsMoneyMarket<'a, T> {
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
    use abstract_core::{
        adapter::AdapterRequestMsg,
        objects::{AnsAsset, AssetEntry},
    };
    use abstract_sdk::mock_module::MockModule;
    use cosmwasm_std::{testing::mock_dependencies, wasm_execute};
    use speculoos::prelude::*;

    use super::*;
    use crate::msg::ExecuteMsg;

    fn expected_request_with_test_proxy(request: MoneyMarketExecuteMsg) -> ExecuteMsg {
        AdapterRequestMsg {
            proxy_address: Some(abstract_testing::prelude::TEST_PROXY.to_string()),
            request,
        }
        .into()
    }

    #[test]
    fn deposit_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let money_market_name = "mars".to_string();
        let asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::AnsAction {
            money_market: money_market_name,
            action: MoneyMarketAnsAction::Deposit {
                lending_asset: asset.clone(),
            },
        });

        let actual = money_market.deposit(asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn withdraw_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let money_market_name = "mars".to_string();
        let asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::AnsAction {
            money_market: money_market_name,
            action: MoneyMarketAnsAction::Withdraw {
                lent_asset: asset.clone(),
            },
        });

        let actual = money_market.withdraw(asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn provide_collateral_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let money_market_name = "mars".to_string();
        let borrowable_asset = AssetEntry::new("usdc");
        let collateral_asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::AnsAction {
            money_market: money_market_name,
            action: MoneyMarketAnsAction::ProvideCollateral {
                borrowable_asset: borrowable_asset.clone(),
                collateral_asset: collateral_asset.clone(),
            },
        });

        let actual = money_market.provide_collateral(collateral_asset, borrowable_asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn withdraw_collateral_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let money_market_name = "mars".to_string();
        let borrowable_asset = AssetEntry::new("usdc");
        let collateral_asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::AnsAction {
            money_market: money_market_name,
            action: MoneyMarketAnsAction::WithdrawCollateral {
                borrowable_asset: borrowable_asset.clone(),
                collateral_asset: collateral_asset.clone(),
            },
        });

        let actual = money_market.withdraw_collateral(collateral_asset, borrowable_asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn borrow_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let money_market_name = "mars".to_string();
        let collateral_asset = AssetEntry::new("juno");
        let borrow_asset = AnsAsset::new("usdc", 1000u128);

        let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::AnsAction {
            money_market: money_market_name,
            action: MoneyMarketAnsAction::Borrow {
                borrow_asset: borrow_asset.clone(),
                collateral_asset: collateral_asset.clone(),
            },
        });

        let actual = money_market.borrow(collateral_asset, borrow_asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn repay_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let money_market = stub
            .ans_money_market(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let money_market_name = "mars".to_string();
        let collateral_asset = AssetEntry::new("juno");
        let borrowed_asset = AnsAsset::new("usdc", 1000u128);

        let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::AnsAction {
            money_market: money_market_name,
            action: MoneyMarketAnsAction::Repay {
                borrowed_asset: borrowed_asset.clone(),
                collateral_asset: collateral_asset.clone(),
            },
        });

        let actual = money_market.repay(collateral_asset, borrowed_asset);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    mod raw {
        use super::*;

        pub const TEST_CONTRACT_ADDR: &str = "test-mm-addr";

        #[test]
        fn deposit_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let money_market_name = "mars".to_string();
            let asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::RawAction {
                money_market: money_market_name,
                action: MoneyMarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneyMarketRawRequest::Deposit {
                        lending_asset: asset.clone().into(),
                    },
                },
            });

            let actual = money_market.deposit(Addr::unchecked(TEST_CONTRACT_ADDR), asset);

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn withdraw_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let money_market_name = "mars".to_string();
            let asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::RawAction {
                money_market: money_market_name,
                action: MoneyMarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneyMarketRawRequest::Withdraw {
                        lent_asset: asset.clone().into(),
                    },
                },
            });

            let actual = money_market.withdraw(Addr::unchecked(TEST_CONTRACT_ADDR), asset);

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn provide_collateral_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let money_market_name = "mars".to_string();
            let borrowable_asset = AssetInfo::native("usdc");
            let collateral_asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::RawAction {
                money_market: money_market_name,
                action: MoneyMarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneyMarketRawRequest::ProvideCollateral {
                        borrowable_asset: borrowable_asset.clone().into(),
                        collateral_asset: collateral_asset.clone().into(),
                    },
                },
            });

            let actual = money_market.provide_collateral(
                Addr::unchecked(TEST_CONTRACT_ADDR),
                collateral_asset,
                borrowable_asset,
            );

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn withdraw_collateral_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let money_market_name = "mars".to_string();
            let borrowable_asset = AssetInfo::native("usdc");
            let collateral_asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::RawAction {
                money_market: money_market_name,
                action: MoneyMarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneyMarketRawRequest::WithdrawCollateral {
                        borrowable_asset: borrowable_asset.clone().into(),
                        collateral_asset: collateral_asset.clone().into(),
                    },
                },
            });

            let actual = money_market.withdraw_collateral(
                Addr::unchecked(TEST_CONTRACT_ADDR),
                collateral_asset,
                borrowable_asset,
            );

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn borrow_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let money_market_name = "mars".to_string();
            let collateral_asset = AssetInfo::native("juno");
            let borrow_asset = Asset::native("usdc", 1000u128);

            let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::RawAction {
                money_market: money_market_name,
                action: MoneyMarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneyMarketRawRequest::Borrow {
                        borrow_asset: borrow_asset.clone().into(),
                        collateral_asset: collateral_asset.clone().into(),
                    },
                },
            });

            let actual = money_market.borrow(
                Addr::unchecked(TEST_CONTRACT_ADDR),
                collateral_asset,
                borrow_asset,
            );

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn repay_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let money_market_name = "mars".to_string();
            let collateral_asset = AssetInfo::native("juno");
            let borrowed_asset = Asset::native("usdc", 1000u128);

            let expected = expected_request_with_test_proxy(MoneyMarketExecuteMsg::RawAction {
                money_market: money_market_name,
                action: MoneyMarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneyMarketRawRequest::Repay {
                        borrowed_asset: borrowed_asset.clone().into(),
                        collateral_asset: collateral_asset.clone().into(),
                    },
                },
            });

            let actual = money_market.repay(
                Addr::unchecked(TEST_CONTRACT_ADDR),
                collateral_asset,
                borrowed_asset,
            );

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }
    }
}
