use crate::MONEYMARKET_ADAPTER_ID;
use abstract_core::objects::{module::ModuleId, AnsAsset, AssetEntry};
use abstract_moneymarket_standard::{
    ans_action::MoneymarketAnsAction,
    msg::{MoneymarketExecuteMsg, MoneymarketName, MoneymarketQueryMsg},
    raw_action::{MoneymarketRawAction, MoneymarketRawRequest},
};
use abstract_sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{Addr, CosmosMsg, Deps};
use cw_asset::{Asset, AssetInfo};

use self::{ans::AnsMoneymarket, raw::MoneyMarket};

// API for Abstract SDK users
/// Interact with the money_market adapter in your module.
pub trait MoneymarketInterface:
    AccountIdentification + Dependencies + ModuleIdentification
{
    /// Construct a new money_market interface.
    fn money_market<'a>(&'a self, deps: Deps<'a>, name: MoneymarketName) -> MoneyMarket<Self> {
        MoneyMarket {
            base: self,
            deps,
            name,
            module_id: MONEYMARKET_ADAPTER_ID,
        }
    }
    /// Construct a new money_market interface with ANS support.
    fn ans_moneymarket<'a>(
        &'a self,
        deps: Deps<'a>,
        name: MoneymarketName,
    ) -> AnsMoneymarket<Self> {
        AnsMoneymarket {
            base: self,
            deps,
            name,
            module_id: MONEYMARKET_ADAPTER_ID,
        }
    }
}

impl<T: AccountIdentification + Dependencies + ModuleIdentification> MoneymarketInterface for T {}

pub mod raw {
    use super::*;

    #[derive(Clone)]
    pub struct MoneyMarket<'a, T: MoneymarketInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: MoneymarketName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
    }

    impl<'a, T: MoneymarketInterface> MoneyMarket<'a, T> {
        /// Set the module id for the MONEYMARKET
        pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
            Self { module_id, ..self }
        }

        /// Use Raw addresses, ids and denoms for money_market-related operations
        pub fn ans(self) -> AnsMoneymarket<'a, T> {
            AnsMoneymarket {
                base: self.base,
                name: self.name,
                module_id: self.module_id,
                deps: self.deps,
            }
        }

        /// returns MONEYMARKET name
        fn moneymarket_name(&self) -> MoneymarketName {
            self.name.clone()
        }

        /// returns the MONEYMARKET module id
        fn moneymarket_module_id(&self) -> ModuleId {
            self.module_id
        }

        /// Executes a [MoneymarketRawAction] in th MONEYMARKET
        fn request(&self, action: MoneymarketRawAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.request(
                self.moneymarket_module_id(),
                MoneymarketExecuteMsg::RawAction {
                    money_market: self.moneymarket_name(),
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
            self.request(MoneymarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneymarketRawRequest::Deposit {
                    lending_asset: lending_asset.into(),
                },
            })
        }

        /// Withdraw liquidity from MONEYMARKET
        pub fn withdraw(
            &self,
            contract_addr: Addr,
            lending_asset: Asset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneymarketRawRequest::Withdraw {
                    lending_asset: lending_asset.into(),
                },
            })
        }

        /// Deposit Collateral in MONEYMARKET
        pub fn provide_collateral(
            &self,
            contract_addr: Addr,
            collateral_asset: Asset,
            borrowed_asset: AssetInfo,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneymarketRawRequest::ProvideCollateral {
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),
                },
            })
        }

        /// Withdraw collateral liquidity from MONEYMARKET
        pub fn withdraw_collateral(
            &self,
            contract_addr: Addr,
            collateral_asset: Asset,
            borrowed_asset: AssetInfo,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneymarketRawRequest::WithdrawCollateral {
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),
                },
            })
        }

        /// Borrow from MoneyMarket
        pub fn borrow(
            &self,
            contract_addr: Addr,
            collateral_asset: AssetInfo,
            borrowed_asset: Asset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneymarketRawRequest::Borrow {
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),
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
            self.request(MoneymarketRawAction {
                contract_addr: contract_addr.to_string(),
                request: MoneymarketRawRequest::Repay {
                    collateral_asset: collateral_asset.into(),
                    borrowed_asset: borrowed_asset.into(),
                },
            })
        }
    }

    impl<'a, T: MoneymarketInterface> MoneyMarket<'a, T> {
        /// Do a query in the MONEYMARKET
        pub fn query<R: DeserializeOwned>(
            &self,
            query_msg: MoneymarketQueryMsg,
        ) -> AbstractSdkResult<R> {
            let adapters = self.base.adapters(self.deps);
            adapters.query(MONEYMARKET_ADAPTER_ID, query_msg)
        }
    }
}

pub mod ans {
    use cosmwasm_schema::serde::de::DeserializeOwned;

    use self::raw::MoneyMarket;

    use super::*;

    #[derive(Clone)]
    pub struct AnsMoneymarket<'a, T: MoneymarketInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: MoneymarketName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
    }

    impl<'a, T: MoneymarketInterface> AnsMoneymarket<'a, T> {
        /// Set the module id for the MONEYMARKET
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

        /// returns MONEYMARKET name
        fn moneymarket_name(&self) -> MoneymarketName {
            self.name.clone()
        }

        /// returns the MONEYMARKET module id
        fn moneymarket_module_id(&self) -> ModuleId {
            self.module_id
        }

        /// Executes a [MoneymarketAction] in th MONEYMARKET
        fn request(&self, action: MoneymarketAnsAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.request(
                self.moneymarket_module_id(),
                MoneymarketExecuteMsg::AnsAction {
                    money_market: self.moneymarket_name(),
                    action,
                },
            )
        }

        /// Deposit assets
        pub fn deposit(&self, lending_asset: AnsAsset) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::Deposit { lending_asset })
        }

        /// Withdraw liquidity from MONEYMARKET
        pub fn withdraw(&self, lending_asset: AnsAsset) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::Withdraw { lending_asset })
        }

        /// Deposit Collateral in MONEYMARKET
        pub fn provide_collateral(
            &self,
            collateral_asset: AnsAsset,
            borrowed_asset: AssetEntry,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::ProvideCollateral {
                collateral_asset,
                borrowed_asset,
            })
        }

        /// Withdraw collateral liquidity from MONEYMARKET
        pub fn withdraw_collateral(
            &self,
            collateral_asset: AnsAsset,
            borrowed_asset: AssetEntry,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::WithdrawCollateral {
                collateral_asset,
                borrowed_asset,
            })
        }

        /// Borrow from MoneyMarket
        pub fn borrow(
            &self,
            collateral_asset: AssetEntry,
            borrowed_asset: AnsAsset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::Borrow {
                collateral_asset,
                borrowed_asset,
            })
        }

        /// Repay borrowed assets from MoneyMarket
        pub fn repay(
            &self,
            collateral_asset: AssetEntry,
            borrowed_asset: AnsAsset,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::Repay {
                collateral_asset,
                borrowed_asset,
            })
        }
    }

    impl<'a, T: MoneymarketInterface> AnsMoneymarket<'a, T> {
        /// Do a query in the MONEYMARKET
        pub fn query<R: DeserializeOwned>(
            &self,
            query_msg: MoneymarketQueryMsg,
        ) -> AbstractSdkResult<R> {
            let adapters = self.base.adapters(self.deps);
            adapters.query(MONEYMARKET_ADAPTER_ID, query_msg)
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

    fn expected_request_with_test_proxy(request: MoneymarketExecuteMsg) -> ExecuteMsg {
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
            .ans_moneymarket(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let moneymarket_name = "mars".to_string();
        let asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            money_market: moneymarket_name,
            action: MoneymarketAnsAction::Deposit {
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
            .ans_moneymarket(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let moneymarket_name = "mars".to_string();
        let asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            money_market: moneymarket_name,
            action: MoneymarketAnsAction::Withdraw {
                lending_asset: asset.clone(),
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
            .ans_moneymarket(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let moneymarket_name = "mars".to_string();
        let borrowed_asset = AssetEntry::new("usdc");
        let collateral_asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            money_market: moneymarket_name,
            action: MoneymarketAnsAction::ProvideCollateral {
                borrowed_asset: borrowed_asset.clone(),
                collateral_asset: collateral_asset.clone(),
            },
        });

        let actual = money_market.provide_collateral(collateral_asset, borrowed_asset);

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
            .ans_moneymarket(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let moneymarket_name = "mars".to_string();
        let borrowed_asset = AssetEntry::new("usdc");
        let collateral_asset = AnsAsset::new("juno", 1000u128);

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            money_market: moneymarket_name,
            action: MoneymarketAnsAction::WithdrawCollateral {
                borrowed_asset: borrowed_asset.clone(),
                collateral_asset: collateral_asset.clone(),
            },
        });

        let actual = money_market.withdraw_collateral(collateral_asset, borrowed_asset);

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
            .ans_moneymarket(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let moneymarket_name = "mars".to_string();
        let collateral_asset = AssetEntry::new("juno");
        let borrowed_asset = AnsAsset::new("usdc", 1000u128);

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            money_market: moneymarket_name,
            action: MoneymarketAnsAction::Borrow {
                borrowed_asset: borrowed_asset.clone(),
                collateral_asset: collateral_asset.clone(),
            },
        });

        let actual = money_market.borrow(collateral_asset, borrowed_asset);

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
            .ans_moneymarket(deps.as_ref(), "mars".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let moneymarket_name = "mars".to_string();
        let collateral_asset = AssetEntry::new("juno");
        let borrowed_asset = AnsAsset::new("usdc", 1000u128);

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            money_market: moneymarket_name,
            action: MoneymarketAnsAction::Repay {
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

            let moneymarket_name = "mars".to_string();
            let asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                money_market: moneymarket_name,
                action: MoneymarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneymarketRawRequest::Deposit {
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

            let moneymarket_name = "mars".to_string();
            let asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                money_market: moneymarket_name,
                action: MoneymarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneymarketRawRequest::Withdraw {
                        lending_asset: asset.clone().into(),
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

            let moneymarket_name = "mars".to_string();
            let borrowed_asset = AssetInfo::native("usdc");
            let collateral_asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                money_market: moneymarket_name,
                action: MoneymarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneymarketRawRequest::ProvideCollateral {
                        borrowed_asset: borrowed_asset.clone().into(),
                        collateral_asset: collateral_asset.clone().into(),
                    },
                },
            });

            let actual = money_market.provide_collateral(
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

        #[test]
        fn withdraw_collateral_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let moneymarket_name = "mars".to_string();
            let borrowed_asset = AssetInfo::native("usdc");
            let collateral_asset = Asset::native("juno", 1000u128);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                money_market: moneymarket_name,
                action: MoneymarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneymarketRawRequest::WithdrawCollateral {
                        borrowed_asset: borrowed_asset.clone().into(),
                        collateral_asset: collateral_asset.clone().into(),
                    },
                },
            });

            let actual = money_market.withdraw_collateral(
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

        #[test]
        fn borrow_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let moneymarket_name = "mars".to_string();
            let collateral_asset = AssetInfo::native("juno");
            let borrowed_asset = Asset::native("usdc", 1000u128);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                money_market: moneymarket_name,
                action: MoneymarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneymarketRawRequest::Borrow {
                        borrowed_asset: borrowed_asset.clone().into(),
                        collateral_asset: collateral_asset.clone().into(),
                    },
                },
            });

            let actual = money_market.borrow(
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

        #[test]
        fn repay_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let money_market = stub
                .money_market(deps.as_ref(), "mars".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let moneymarket_name = "mars".to_string();
            let collateral_asset = AssetInfo::native("juno");
            let borrowed_asset = Asset::native("usdc", 1000u128);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                money_market: moneymarket_name,
                action: MoneymarketRawAction {
                    contract_addr: TEST_CONTRACT_ADDR.to_string(),
                    request: MoneymarketRawRequest::Repay {
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
