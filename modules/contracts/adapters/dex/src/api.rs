use crate::DEX_ADAPTER_ID;
use abstract_adapter::sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
};
use abstract_adapter::std::objects::{module::ModuleId, AnsAsset, AssetEntry, PoolAddress};
use abstract_dex_standard::msg::GenerateMessagesResponse;
use abstract_dex_standard::{
    ans_action::DexAnsAction,
    msg::{DexExecuteMsg, DexName, DexQueryMsg, SimulateSwapResponse},
    raw_action::DexRawAction,
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{CosmosMsg, Decimal, Deps};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

use self::{ans::AnsDex, raw::Dex};

// API for Abstract SDK users
/// Interact with the dex adapter in your module.
pub trait DexInterface: AccountIdentification + Dependencies + ModuleIdentification {
    /// Construct a new dex interface.
    fn dex<'a>(&'a self, deps: Deps<'a>, name: DexName) -> Dex<Self> {
        Dex {
            base: self,
            deps,
            name,
            module_id: DEX_ADAPTER_ID,
        }
    }
    /// Construct a new dex interface with ANS support.
    fn ans_dex<'a>(&'a self, deps: Deps<'a>, name: DexName) -> AnsDex<Self> {
        AnsDex {
            base: self,
            deps,
            name,
            module_id: DEX_ADAPTER_ID,
        }
    }
}

impl<T: AccountIdentification + Dependencies + ModuleIdentification> DexInterface for T {}

pub mod raw {
    use super::*;

    #[derive(Clone)]
    pub struct Dex<'a, T: DexInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: DexName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
    }

    impl<'a, T: DexInterface> Dex<'a, T> {
        /// Set the module id for the DEX
        pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
            Self { module_id, ..self }
        }

        /// Use Raw addresses, ids and denoms for dex-related operations
        pub fn ans(self) -> AnsDex<'a, T> {
            AnsDex {
                base: self.base,
                name: self.name,
                module_id: self.module_id,
                deps: self.deps,
            }
        }

        /// returns DEX name
        fn dex_name(&self) -> DexName {
            self.name.clone()
        }

        /// returns the DEX module id
        fn dex_module_id(&self) -> ModuleId {
            self.module_id
        }

        /// Executes a [DexRawAction] in th DEX
        fn execute(&self, action: DexRawAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.execute(
                self.dex_module_id(),
                DexExecuteMsg::RawAction {
                    dex: self.dex_name(),
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
            self.execute(DexRawAction::Swap {
                offer_asset: offer_asset.into(),
                ask_asset: ask_asset.into(),
                belief_price,
                max_spread,
                pool: pool.into(),
            })
        }

        /// Provide liquidity in the DEX
        pub fn provide_liquidity(
            &self,
            assets: Vec<Asset>,
            max_spread: Option<Decimal>,
            pool: PoolAddress,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(DexRawAction::ProvideLiquidity {
                assets: assets.into_iter().map(Into::into).collect(),
                pool: pool.into(),
                max_spread,
            })
        }

        /// Provide symmetric liquidity in the DEX
        pub fn provide_liquidity_symmetric(
            &self,
            offer_asset: Asset,
            paired_assets: Vec<AssetInfo>,
            pool: PoolAddress,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(DexRawAction::ProvideLiquiditySymmetric {
                offer_asset: offer_asset.into(),
                paired_assets: paired_assets.into_iter().map(Into::into).collect(),
                pool: pool.into(),
            })
        }

        /// Withdraw liquidity from the DEX
        pub fn withdraw_liquidity(
            &self,
            lp_token: Asset,
            pool: PoolAddress,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(DexRawAction::WithdrawLiquidity {
                lp_token: lp_token.into(),
                pool: pool.into(),
            })
        }
    }

    impl<'a, T: DexInterface> Dex<'a, T> {
        /// Do a query in the DEX
        fn query<R: DeserializeOwned>(&self, query_msg: DexQueryMsg) -> AbstractSdkResult<R> {
            let adapters = self.base.adapters(self.deps);
            adapters.query(DEX_ADAPTER_ID, query_msg)
        }

        /// simulate DEx swap without relying on ANS
        pub fn simulate_swap(
            &self,
            offer_asset: Asset,
            ask_asset: AssetInfo,
            pool: PoolAddress,
        ) -> AbstractSdkResult<SimulateSwapResponse<AssetInfoBase<String>>> {
            let response: SimulateSwapResponse<AssetInfoBase<String>> =
                self.query(DexQueryMsg::SimulateSwapRaw {
                    offer_asset: offer_asset.into(),
                    ask_asset: ask_asset.into(),
                    pool: pool.into(),
                    dex: self.dex_name(),
                })?;
            Ok(response)
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
            let response: GenerateMessagesResponse = self.query(DexQueryMsg::GenerateMessages {
                message: DexExecuteMsg::RawAction {
                    dex: self.dex_name(),
                    action: DexRawAction::Swap {
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
    use super::*;

    #[derive(Clone)]
    pub struct AnsDex<'a, T: DexInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: DexName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
    }

    impl<'a, T: DexInterface> AnsDex<'a, T> {
        /// Set the module id for the DEX
        pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
            Self { module_id, ..self }
        }

        /// Use Raw addresses, ids and denoms for dex-related operations
        pub fn raw(self) -> Dex<'a, T> {
            Dex {
                base: self.base,
                name: self.name,
                module_id: self.module_id,
                deps: self.deps,
            }
        }

        /// returns DEX name
        fn dex_name(&self) -> DexName {
            self.name.clone()
        }

        /// returns the DEX module id
        fn dex_module_id(&self) -> ModuleId {
            self.module_id
        }

        /// Executes a [DexAction] in th DEX
        fn execute(&self, action: DexAnsAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.execute(
                self.dex_module_id(),
                DexExecuteMsg::AnsAction {
                    dex: self.dex_name(),
                    action,
                },
            )
        }

        /// Swap assets in the DEX
        pub fn swap(
            &self,
            offer_asset: AnsAsset,
            ask_asset: AssetEntry,
            max_spread: Option<Decimal>,
            belief_price: Option<Decimal>,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(DexAnsAction::Swap {
                offer_asset,
                ask_asset,
                belief_price,
                max_spread,
            })
        }

        /// Provide liquidity in the DEX
        pub fn provide_liquidity(
            &self,
            assets: Vec<AnsAsset>,
            max_spread: Option<Decimal>,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(DexAnsAction::ProvideLiquidity { assets, max_spread })
        }

        /// Provide symmetrict liquidity in the DEX
        pub fn provide_liquidity_symmetric(
            &self,
            offer_asset: AnsAsset,
            paired_assets: Vec<AssetEntry>,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(DexAnsAction::ProvideLiquiditySymmetric {
                offer_asset,
                paired_assets,
            })
        }

        /// Withdraw liquidity from the DEX
        pub fn withdraw_liquidity(&self, lp_token: AnsAsset) -> AbstractSdkResult<CosmosMsg> {
            self.execute(DexAnsAction::WithdrawLiquidity { lp_token })
        }
    }

    impl<'a, T: DexInterface> AnsDex<'a, T> {
        /// Do a query in the DEX
        fn query<R: DeserializeOwned>(&self, query_msg: DexQueryMsg) -> AbstractSdkResult<R> {
            let adapters = self.base.adapters(self.deps);
            adapters.query(DEX_ADAPTER_ID, query_msg)
        }

        /// simulate DEx swap
        pub fn simulate_swap(
            &self,
            offer_asset: AnsAsset,
            ask_asset: AssetEntry,
        ) -> AbstractSdkResult<SimulateSwapResponse> {
            let response: SimulateSwapResponse = self.query(DexQueryMsg::SimulateSwap {
                dex: self.dex_name(),
                offer_asset,
                ask_asset,
            })?;
            Ok(response)
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
            let response: GenerateMessagesResponse = self.query(DexQueryMsg::GenerateMessages {
                message: DexExecuteMsg::AnsAction {
                    dex: self.dex_name(),
                    action: DexAnsAction::Swap {
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
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_adapter::sdk::mock_module::MockModule;
    use abstract_adapter::std::adapter::AdapterRequestMsg;
    use cosmwasm_std::{testing::mock_dependencies, wasm_execute};
    use speculoos::prelude::*;

    use super::*;
    use crate::msg::ExecuteMsg;

    fn expected_request_with_test_proxy(request: DexExecuteMsg) -> ExecuteMsg {
        AdapterRequestMsg {
            proxy_address: Some(
                abstract_adapter::abstract_testing::prelude::TEST_PROXY.to_string(),
            ),
            request,
        }
        .into()
    }

    #[test]
    fn swap_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_adapter::abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex = stub
            .ans_dex(deps.as_ref(), "junoswap".into())
            .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

        let dex_name = "junoswap".to_string();
        let offer_asset = AnsAsset::new("juno", 1000u128);
        let ask_asset = AssetEntry::new("uusd");
        let max_spread = Some(Decimal::percent(1));
        let belief_price = Some(Decimal::percent(2));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::AnsAction {
            dex: dex_name,
            action: DexAnsAction::Swap {
                offer_asset: offer_asset.clone(),
                ask_asset: ask_asset.clone(),
                max_spread,
                belief_price,
            },
        });

        let actual = dex.swap(offer_asset, ask_asset, max_spread, belief_price);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_adapter::abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn provide_liquidity_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_adapter::abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .ans_dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

        let assets = vec![AnsAsset::new("taco", 1000u128)];
        let max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::AnsAction {
            dex: dex_name,
            action: DexAnsAction::ProvideLiquidity {
                assets: assets.clone(),
                max_spread,
            },
        });

        let actual = dex.provide_liquidity(assets, max_spread);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_adapter::abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn provide_liquidity_symmetric_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_adapter::abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .ans_dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

        let offer = AnsAsset::new("taco", 1000u128);
        let paired = vec![AssetEntry::new("bell")];
        let _max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::AnsAction {
            dex: dex_name,
            action: DexAnsAction::ProvideLiquiditySymmetric {
                offer_asset: offer.clone(),
                paired_assets: paired.clone(),
            },
        });

        let actual = dex.provide_liquidity_symmetric(offer, paired);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_adapter::abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn withdraw_liquidity_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_adapter::abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .ans_dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

        let lp_token = AnsAsset::new("taco", 1000u128);

        let expected = expected_request_with_test_proxy(DexExecuteMsg::AnsAction {
            dex: dex_name,
            action: DexAnsAction::WithdrawLiquidity {
                lp_token: lp_token.clone(),
            },
        });

        let actual = dex.withdraw_liquidity(lp_token);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_adapter::abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    mod raw {
        use abstract_adapter::std::objects::pool_id::PoolAddressBase;

        use super::*;

        pub const POOL: u64 = 1278734;

        #[test]
        fn swap_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_adapter::abstract_testing::mock_querier();
            let stub = MockModule::new();
            let dex = stub
                .dex(deps.as_ref(), "junoswap".into())
                .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

            let dex_name = "junoswap".to_string();
            let offer_asset = Asset::native("ujuno", 100_000u128);
            let ask_asset = AssetInfo::native("uusd");
            let max_spread = Some(Decimal::percent(1));
            let belief_price = Some(Decimal::percent(2));
            let pool = PoolAddressBase::Id(POOL);

            let expected = expected_request_with_test_proxy(DexExecuteMsg::RawAction {
                dex: dex_name,
                action: DexRawAction::Swap {
                    offer_asset: offer_asset.clone().into(),
                    ask_asset: ask_asset.clone().into(),
                    max_spread,
                    belief_price,
                    pool: pool.clone().into(),
                },
            });

            let actual = dex.swap(offer_asset, ask_asset, max_spread, belief_price, pool);

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_adapter::abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn provide_liquidity_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_adapter::abstract_testing::mock_querier();
            let stub = MockModule::new();
            let dex_name = "junoswap".to_string();

            let dex = stub
                .dex(deps.as_ref(), dex_name.clone())
                .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

            let assets = vec![Asset::native("taco", 1000u128)];
            let max_spread = Some(Decimal::percent(1));
            let pool = PoolAddressBase::Id(POOL);

            let expected = expected_request_with_test_proxy(DexExecuteMsg::RawAction {
                dex: dex_name,
                action: DexRawAction::ProvideLiquidity {
                    assets: assets.clone().into_iter().map(Into::into).collect(),
                    max_spread,
                    pool: pool.clone().into(),
                },
            });

            let actual = dex.provide_liquidity(assets, max_spread, pool);

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_adapter::abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn provide_liquidity_symmetric_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_adapter::abstract_testing::mock_querier();
            let stub = MockModule::new();
            let dex_name = "junoswap".to_string();

            let dex = stub
                .dex(deps.as_ref(), dex_name.clone())
                .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

            let offer = Asset::native("taco", 1000u128);
            let paired = vec![AssetInfo::native("bell")];
            let _max_spread = Some(Decimal::percent(1));
            let pool = PoolAddressBase::Id(POOL);

            let expected = expected_request_with_test_proxy(DexExecuteMsg::RawAction {
                dex: dex_name,
                action: DexRawAction::ProvideLiquiditySymmetric {
                    offer_asset: offer.clone().into(),
                    paired_assets: paired.clone().into_iter().map(Into::into).collect(),
                    pool: pool.clone().into(),
                },
            });

            let actual = dex.provide_liquidity_symmetric(offer, paired, pool);

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_adapter::abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn withdraw_liquidity_msg() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_adapter::abstract_testing::mock_querier();
            let stub = MockModule::new();
            let dex_name = "junoswap".to_string();

            let dex = stub
                .dex(deps.as_ref(), dex_name.clone())
                .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

            let lp_token = Asset::native("taco", 1000u128);
            let pool = PoolAddressBase::Id(POOL);

            let expected = expected_request_with_test_proxy(DexExecuteMsg::RawAction {
                dex: dex_name,
                action: DexRawAction::WithdrawLiquidity {
                    lp_token: lp_token.clone().into(),
                    pool: pool.clone().into(),
                },
            });

            let actual = dex.withdraw_liquidity(lp_token, pool);

            assert_that!(actual).is_ok();

            let actual = match actual.unwrap() {
                CosmosMsg::Wasm(msg) => msg,
                _ => panic!("expected wasm msg"),
            };
            let expected = wasm_execute(
                abstract_adapter::abstract_testing::prelude::TEST_MODULE_ADDRESS,
                &expected,
                vec![],
            )
            .unwrap();

            assert_that!(actual).is_equal_to(expected);
        }
    }
}
