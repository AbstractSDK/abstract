// TODO: this should be moved to the public dex package
// It cannot be in abstract-os because it does not have a dependency on sdk (as it shouldn't)
use abstract_core::objects::{module::ModuleId, AssetEntry};
use abstract_dex_standard::msg::{
    DexAction, DexExecuteMsg, DexName, DexQueryMsg, OfferAsset, SimulateSwapResponse,
};
use abstract_sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
};
use cosmwasm_std::{CosmosMsg, Decimal, Deps, Uint128};
use serde::de::DeserializeOwned;

use crate::DEX_ADAPTER_ID;

// API for Abstract SDK users
/// Interact with the dex adapter in your module.
pub trait DexInterface: AccountIdentification + Dependencies + ModuleIdentification {
    /// Construct a new dex interface
    fn dex<'a>(&'a self, deps: Deps<'a>, name: DexName) -> Dex<Self> {
        Dex {
            base: self,
            deps,
            name,
            module_id: DEX_ADAPTER_ID,
        }
    }
}

impl<T: AccountIdentification + Dependencies + ModuleIdentification> DexInterface for T {}

#[derive(Clone)]
pub struct Dex<'a, T: DexInterface> {
    base: &'a T,
    name: DexName,
    module_id: ModuleId<'a>,
    deps: Deps<'a>,
}

impl<'a, T: DexInterface> Dex<'a, T> {
    /// Set the module id for the DEX
    pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
        Self { module_id, ..self }
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
    fn request(&self, action: DexAction) -> AbstractSdkResult<CosmosMsg> {
        let adapters = self.base.adapters(self.deps);

        adapters.request(
            self.dex_module_id(),
            DexExecuteMsg::Action {
                dex: self.dex_name(),
                action,
            },
        )
    }

    /// Swap assets in the DEX
    pub fn swap(
        &self,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::Swap {
            offer_asset,
            ask_asset,
            belief_price,
            max_spread,
        })
    }

    /// Provide liquidity in the DEX
    pub fn provide_liquidity(
        &self,
        assets: Vec<OfferAsset>,
        max_spread: Option<Decimal>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::ProvideLiquidity { assets, max_spread })
    }

    /// Provide symmetrict liquidity in the DEX
    pub fn provide_liquidity_symmetric(
        &self,
        offer_asset: OfferAsset,
        paired_assets: Vec<AssetEntry>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::ProvideLiquiditySymmetric {
            offer_asset,
            paired_assets,
        })
    }

    /// Withdraw liquidity from the DEX
    pub fn withdraw_liquidity(
        &self,
        lp_token: AssetEntry,
        amount: Uint128,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::WithdrawLiquidity { lp_token, amount })
    }
}

impl<'a, T: DexInterface> Dex<'a, T> {
    /// Do a query in the DEX
    pub fn query<R: DeserializeOwned>(&self, query_msg: DexQueryMsg) -> AbstractSdkResult<R> {
        let adapters = self.base.adapters(self.deps);
        adapters.query(DEX_ADAPTER_ID, query_msg)
    }

    /// simulate DEx swap
    pub fn simulate_swap(
        &self,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
    ) -> AbstractSdkResult<SimulateSwapResponse> {
        let response: SimulateSwapResponse = self.query(DexQueryMsg::SimulateSwap {
            dex: Some(self.dex_name()),
            offer_asset,
            ask_asset,
        })?;
        Ok(response)
    }

    /// Generate the raw messages that are need to run a swap
    pub fn generate_swap_messages(
        &self,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
        sender_receiver: impl Into<String>,
    ) -> AbstractSdkResult<SimulateSwapResponse> {
        let response: SimulateSwapResponse = self.query(DexQueryMsg::GenerateMessages {
            message: DexExecuteMsg::Action {
                dex: self.dex_name(),
                action: DexAction::Swap {
                    offer_asset,
                    ask_asset,
                    max_spread,
                    belief_price,
                },
            },
            sender: sender_receiver.into(),
        })?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use abstract_core::adapter::AdapterRequestMsg;
    use abstract_sdk::mock_module::MockModule;
    use cosmwasm_std::{testing::mock_dependencies, wasm_execute};
    use speculoos::prelude::*;

    use super::*;
    use crate::msg::ExecuteMsg;

    fn expected_request_with_test_proxy(request: DexExecuteMsg) -> ExecuteMsg {
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
        let dex = stub
            .dex(deps.as_ref(), "junoswap".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let dex_name = "junoswap".to_string();
        let offer_asset = OfferAsset::new("juno", 1000u128);
        let ask_asset = AssetEntry::new("uusd");
        let max_spread = Some(Decimal::percent(1));
        let belief_price = Some(Decimal::percent(2));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::Swap {
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
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let assets = vec![OfferAsset::new("taco", 1000u128)];
        let max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::ProvideLiquidity {
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
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let offer = OfferAsset::new("taco", 1000u128);
        let paired = vec![AssetEntry::new("bell")];
        let _max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::ProvideLiquiditySymmetric {
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
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let lp_token = AssetEntry::new("taco");
        let withdraw_amount: Uint128 = 1000u128.into();

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::WithdrawLiquidity {
                lp_token: lp_token.clone(),
                amount: withdraw_amount,
            },
        });

        let actual = dex.withdraw_liquidity(lp_token, withdraw_amount);

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
