use crate::MONEYMARKET_ADAPTER_ID;
use abstract_core::objects::{module::ModuleId, AnsAsset, AssetEntry, PoolAddress};
use abstract_moneymarket_standard::msg::GenerateMessagesResponse;
use abstract_moneymarket_standard::{
    ans_action::MoneymarketAnsAction,
    msg::{MoneymarketExecuteMsg, MoneymarketName, MoneymarketQueryMsg},
    raw_action::MoneymarketRawAction,
};
use abstract_sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{CosmosMsg, Decimal, Deps};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

use self::{ans::AnsMoneymarket, raw::Moneymarket};

// API for Abstract SDK users
/// Interact with the moneymarket adapter in your module.
pub trait MoneymarketInterface:
    AccountIdentification + Dependencies + ModuleIdentification
{
    /// Construct a new moneymarket interface.
    fn moneymarket<'a>(&'a self, deps: Deps<'a>, name: MoneymarketName) -> Moneymarket<Self> {
        Moneymarket {
            base: self,
            deps,
            name,
            module_id: MONEYMARKET_ADAPTER_ID,
        }
    }
    /// Construct a new moneymarket interface with ANS support.
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
    pub struct Moneymarket<'a, T: MoneymarketInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: MoneymarketName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
    }

    impl<'a, T: MoneymarketInterface> Moneymarket<'a, T> {
        /// Set the module id for the MONEYMARKET
        pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
            Self { module_id, ..self }
        }

        /// Use Raw addresses, ids and denoms for moneymarket-related operations
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
                    moneymarket: self.moneymarket_name(),
                    action,
                },
            )
        }

        /// Swap assets without ANS
        pub fn swap(
            &self,
            offer_asset: Asset,
            ask_asset: AssetInfo,
            max_spread: Option<Decimal>,
            belief_price: Option<Decimal>,
            pool: PoolAddress,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketRawAction::Swap {
                offer_asset: offer_asset.into(),
                ask_asset: ask_asset.into(),
                belief_price,
                max_spread,
                pool: pool.into(),
            })
        }

        /// Provide liquidity in the MONEYMARKET
        pub fn provide_liquidity(
            &self,
            assets: Vec<Asset>,
            max_spread: Option<Decimal>,
            pool: PoolAddress,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketRawAction::ProvideLiquidity {
                assets: assets.into_iter().map(Into::into).collect(),
                pool: pool.into(),
                max_spread,
            })
        }

        /// Provide symmetric liquidity in the MONEYMARKET
        pub fn provide_liquidity_symmetric(
            &self,
            offer_asset: Asset,
            paired_assets: Vec<AssetInfo>,
            pool: PoolAddress,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketRawAction::ProvideLiquiditySymmetric {
                offer_asset: offer_asset.into(),
                paired_assets: paired_assets.into_iter().map(Into::into).collect(),
                pool: pool.into(),
            })
        }

        /// Withdraw liquidity from the MONEYMARKET
        pub fn withdraw_liquidity(
            &self,
            lp_token: Asset,
            pool: PoolAddress,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketRawAction::WithdrawLiquidity {
                lp_token: lp_token.into(),
                pool: pool.into(),
            })
        }
    }

    impl<'a, T: MoneymarketInterface> Moneymarket<'a, T> {
        /// Do a query in the MONEYMARKET
        fn query<R: DeserializeOwned>(
            &self,
            query_msg: MoneymarketQueryMsg,
        ) -> AbstractSdkResult<R> {
            let adapters = self.base.adapters(self.deps);
            adapters.query(MONEYMARKET_ADAPTER_ID, query_msg)
        }

        /// Generate the raw messages that are need to run a swap
        pub fn generate_swap_messages(
            &self,
            offer_asset: Asset,
            ask_asset: AssetInfo,
            pool: PoolAddress,
            max_spread: Option<Decimal>,
            belief_price: Option<Decimal>,
            addr_as_sender: impl Into<String>,
        ) -> AbstractSdkResult<GenerateMessagesResponse> {
            let response: GenerateMessagesResponse =
                self.query(MoneymarketQueryMsg::GenerateMessages {
                    message: MoneymarketExecuteMsg::RawAction {
                        moneymarket: self.moneymarket_name(),
                        action: MoneymarketRawAction::Swap {
                            offer_asset: offer_asset.into(),
                            ask_asset: ask_asset.into(),
                            max_spread,
                            belief_price,
                            pool: pool.into(),
                        },
                    },
                    addr_as_sender: addr_as_sender.into(),
                })?;
            Ok(response)
        }
    }
}

pub mod ans {
    use cosmwasm_schema::serde::de::DeserializeOwned;

    use self::raw::Moneymarket;

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

        /// Use Raw addresses, ids and denoms for moneymarket-related operations
        pub fn raw(self) -> Moneymarket<'a, T> {
            Moneymarket {
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
                    moneymarket: self.moneymarket_name(),
                    action,
                },
            )
        }

        /// Swap assets in the MONEYMARKET
        pub fn swap(
            &self,
            offer_asset: AnsAsset,
            ask_asset: AssetEntry,
            max_spread: Option<Decimal>,
            belief_price: Option<Decimal>,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::Swap {
                offer_asset,
                ask_asset,
                belief_price,
                max_spread,
            })
        }

        /// Provide liquidity in the MONEYMARKET
        pub fn provide_liquidity(
            &self,
            assets: Vec<AnsAsset>,
            max_spread: Option<Decimal>,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::ProvideLiquidity { assets, max_spread })
        }

        /// Provide symmetrict liquidity in the MONEYMARKET
        pub fn provide_liquidity_symmetric(
            &self,
            offer_asset: AnsAsset,
            paired_assets: Vec<AssetEntry>,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::ProvideLiquiditySymmetric {
                offer_asset,
                paired_assets,
            })
        }

        /// Withdraw liquidity from the MONEYMARKET
        pub fn withdraw_liquidity(&self, lp_token: AnsAsset) -> AbstractSdkResult<CosmosMsg> {
            self.request(MoneymarketAnsAction::WithdrawLiquidity { lp_token })
        }
    }

    impl<'a, T: MoneymarketInterface> AnsMoneymarket<'a, T> {
        /// Do a query in the MONEYMARKET
        fn query<R: DeserializeOwned>(
            &self,
            query_msg: MoneymarketQueryMsg,
        ) -> AbstractSdkResult<R> {
            let adapters = self.base.adapters(self.deps);
            adapters.query(MONEYMARKET_ADAPTER_ID, query_msg)
        }

        /// Generate the raw messages that are need to run a swap
        pub fn generate_swap_messages(
            &self,
            offer_asset: AnsAsset,
            ask_asset: AssetEntry,
            max_spread: Option<Decimal>,
            belief_price: Option<Decimal>,
            addr_as_sender: impl Into<String>,
        ) -> AbstractSdkResult<GenerateMessagesResponse> {
            let response: GenerateMessagesResponse =
                self.query(MoneymarketQueryMsg::GenerateMessages {
                    message: MoneymarketExecuteMsg::AnsAction {
                        moneymarket: self.moneymarket_name(),
                        action: MoneymarketAnsAction::Swap {
                            offer_asset,
                            ask_asset,
                            max_spread,
                            belief_price,
                        },
                    },
                    addr_as_sender: addr_as_sender.into(),
                })?;
            Ok(response)
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
    fn swap_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let moneymarket = stub
            .ans_moneymarket(deps.as_ref(), "junoswap".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let moneymarket_name = "junoswap".to_string();
        let offer_asset = AnsAsset::new("juno", 1000u128);
        let ask_asset = AssetEntry::new("uusd");
        let max_spread = Some(Decimal::percent(1));
        let belief_price = Some(Decimal::percent(2));

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            moneymarket: moneymarket_name,
            action: MoneymarketAnsAction::Swap {
                offer_asset: offer_asset.clone(),
                ask_asset: ask_asset.clone(),
                max_spread,
                belief_price,
            },
        });

        let actual = moneymarket.swap(offer_asset, ask_asset, max_spread, belief_price);

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
    fn provide_liquidity_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let moneymarket_name = "junoswap".to_string();

        let moneymarket = stub
            .ans_moneymarket(deps.as_ref(), moneymarket_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let assets = vec![AnsAsset::new("taco", 1000u128)];
        let max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            moneymarket: moneymarket_name,
            action: MoneymarketAnsAction::ProvideLiquidity {
                assets: assets.clone(),
                max_spread,
            },
        });

        let actual = moneymarket.provide_liquidity(assets, max_spread);

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
    fn provide_liquidity_symmetric_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let moneymarket_name = "junoswap".to_string();

        let moneymarket = stub
            .ans_moneymarket(deps.as_ref(), moneymarket_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let offer = AnsAsset::new("taco", 1000u128);
        let paired = vec![AssetEntry::new("bell")];
        let _max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            moneymarket: moneymarket_name,
            action: MoneymarketAnsAction::ProvideLiquiditySymmetric {
                offer_asset: offer.clone(),
                paired_assets: paired.clone(),
            },
        });

        let actual = moneymarket.provide_liquidity_symmetric(offer, paired);

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
    fn withdraw_liquidity_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let moneymarket_name = "junoswap".to_string();

        let moneymarket = stub
            .ans_moneymarket(deps.as_ref(), moneymarket_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let lp_token = AnsAsset::new("taco", 1000u128);

        let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::AnsAction {
            moneymarket: moneymarket_name,
            action: MoneymarketAnsAction::WithdrawLiquidity {
                lp_token: lp_token.clone(),
            },
        });

        let actual = moneymarket.withdraw_liquidity(lp_token);

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
        use abstract_core::objects::pool_id::PoolAddressBase;

        use super::*;

        pub const POOL: u64 = 1278734;

        #[test]
        fn swap_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let moneymarket = stub
                .moneymarket(deps.as_ref(), "junoswap".into())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let moneymarket_name = "junoswap".to_string();
            let offer_asset = Asset::native("ujuno", 100_000u128);
            let ask_asset = AssetInfo::native("uusd");
            let max_spread = Some(Decimal::percent(1));
            let belief_price = Some(Decimal::percent(2));
            let pool = PoolAddressBase::Id(POOL);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                moneymarket: moneymarket_name,
                action: MoneymarketRawAction::Swap {
                    offer_asset: offer_asset.clone().into(),
                    ask_asset: ask_asset.clone().into(),
                    max_spread,
                    belief_price,
                    pool: pool.clone().into(),
                },
            });

            let actual = moneymarket.swap(offer_asset, ask_asset, max_spread, belief_price, pool);

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
        fn provide_liquidity_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let moneymarket_name = "junoswap".to_string();

            let moneymarket = stub
                .moneymarket(deps.as_ref(), moneymarket_name.clone())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let assets = vec![Asset::native("taco", 1000u128)];
            let max_spread = Some(Decimal::percent(1));
            let pool = PoolAddressBase::Id(POOL);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                moneymarket: moneymarket_name,
                action: MoneymarketRawAction::ProvideLiquidity {
                    assets: assets.clone().into_iter().map(Into::into).collect(),
                    max_spread,
                    pool: pool.clone().into(),
                },
            });

            let actual = moneymarket.provide_liquidity(assets, max_spread, pool);

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
        fn provide_liquidity_symmetric_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let moneymarket_name = "junoswap".to_string();

            let moneymarket = stub
                .moneymarket(deps.as_ref(), moneymarket_name.clone())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let offer = Asset::native("taco", 1000u128);
            let paired = vec![AssetInfo::native("bell")];
            let _max_spread = Some(Decimal::percent(1));
            let pool = PoolAddressBase::Id(POOL);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                moneymarket: moneymarket_name,
                action: MoneymarketRawAction::ProvideLiquiditySymmetric {
                    offer_asset: offer.clone().into(),
                    paired_assets: paired.clone().into_iter().map(Into::into).collect(),
                    pool: pool.clone().into(),
                },
            });

            let actual = moneymarket.provide_liquidity_symmetric(offer, paired, pool);

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
        fn withdraw_liquidity_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_testing::mock_querier();
            let stub = MockModule::new();
            let moneymarket_name = "junoswap".to_string();

            let moneymarket = stub
                .moneymarket(deps.as_ref(), moneymarket_name.clone())
                .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

            let lp_token = Asset::native("taco", 1000u128);
            let pool = PoolAddressBase::Id(POOL);

            let expected = expected_request_with_test_proxy(MoneymarketExecuteMsg::RawAction {
                moneymarket: moneymarket_name,
                action: MoneymarketRawAction::WithdrawLiquidity {
                    lp_token: lp_token.clone().into(),
                    pool: pool.clone().into(),
                },
            });

            let actual = moneymarket.withdraw_liquidity(lp_token, pool);

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
