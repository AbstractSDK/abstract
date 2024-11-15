use crate::DEX_ADAPTER_ID;
use abstract_adapter::sdk::Resolve;
use abstract_adapter::sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
};
use abstract_adapter::std::objects::{module::ModuleId, AnsAsset, AssetEntry, PoolAddress};

use abstract_adapter::traits::AbstractNameService;
use abstract_dex_standard::msg::GenerateMessagesResponse;
use abstract_dex_standard::{
    action::DexAction,
    msg::{DexExecuteMsg, DexName, DexQueryMsg, SimulateSwapResponse},
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{CosmosMsg, Decimal, Deps, Env};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

use self::{ans::AnsDex, raw::Dex};

// API for Abstract SDK users
/// Interact with the dex adapter in your module.
pub trait DexInterface:
    AccountIdentification + Dependencies + ModuleIdentification + AbstractNameService
{
    /// Construct a new dex interface.
    fn dex<'a>(&'a self, deps: Deps<'a>, env: &'a Env, name: DexName) -> Dex<Self> {
        Dex {
            base: self,
            env,
            deps,
            name,
            module_id: DEX_ADAPTER_ID,
        }
    }
    /// Construct a new dex interface with ANS support.
    fn ans_dex<'a>(&'a self, deps: Deps<'a>, env: &'a Env, name: DexName) -> AnsDex<Self> {
        AnsDex {
            base: self,
            env,
            deps,
            name,
            module_id: DEX_ADAPTER_ID,
        }
    }
}

impl<T: AccountIdentification + Dependencies + ModuleIdentification + AbstractNameService>
    DexInterface for T
{
}

pub mod raw {
    use super::*;

    #[derive(Clone)]
    pub struct Dex<'a, T: DexInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: DexName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
        pub(crate) env: &'a Env,
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
                env: self.env,
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
        fn execute(&self, action: DexAction) -> AbstractSdkResult<CosmosMsg> {
            let adapters = self.base.adapters(self.deps);

            adapters.execute(
                self.dex_module_id(),
                DexExecuteMsg::Action {
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
            self.execute(DexAction::Swap {
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
            self.execute(DexAction::ProvideLiquidity {
                assets: assets.into_iter().map(Into::into).collect(),
                pool: pool.into(),
                max_spread,
            })
        }

        /// Withdraw liquidity from the DEX
        pub fn withdraw_liquidity(
            &self,
            lp_token: Asset,
            pool: PoolAddress,
        ) -> AbstractSdkResult<CosmosMsg> {
            self.execute(DexAction::WithdrawLiquidity {
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
                message: DexExecuteMsg::Action {
                    dex: self.dex_name(),
                    action: DexAction::Swap {
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
    use abstract_adapter::std::AbstractError;
    use abstract_dex_standard::ans_action::{DexAnsAction, WholeDexAction};

    use super::*;

    #[derive(Clone)]
    pub struct AnsDex<'a, T: DexInterface> {
        pub(crate) base: &'a T,
        pub(crate) name: DexName,
        pub(crate) module_id: ModuleId<'a>,
        pub(crate) deps: Deps<'a>,
        pub(crate) env: &'a Env,
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
                env: self.env,
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
            let ans_host = self.base.ans_host(self.deps)?;
            let message = WholeDexAction(self.dex_name(), action)
                .resolve(&self.deps.querier, &ans_host)
                .map_err(AbstractError::from)?;

            adapters.execute(self.dex_module_id(), message)
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
            let ans_host = self.base.ans_host(self.deps)?;
            let message = WholeDexAction(
                self.dex_name(),
                DexAnsAction::Swap {
                    offer_asset,
                    ask_asset,
                    max_spread,
                    belief_price,
                },
            )
            .resolve(&self.deps.querier, &ans_host)
            .map_err(AbstractError::from)?;

            let response: GenerateMessagesResponse = self.query(DexQueryMsg::GenerateMessages {
                message,
                addr_as_sender: addr_as_sender.into(),
            })?;
            Ok(response)
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;
    use crate::msg::ExecuteMsg;
    use abstract_adapter::abstract_testing::mock_env_validated;
    use abstract_adapter::abstract_testing::prelude::{
        test_account, AbstractMockQuerier, TEST_ACCOUNT_ID,
    };
    use abstract_adapter::std::adapter::AdapterRequestMsg;
    use abstract_adapter::std::objects::pool_id::PoolAddressBase;
    use abstract_adapter::{
        abstract_testing::prelude::AbstractMockAddrs, sdk::mock_module::MockModule,
    };
    use cosmwasm_std::{testing::mock_dependencies, wasm_execute, Addr};

    pub const POOL: u64 = 1278734;

    fn expected_request_with_test_account(
        request: DexExecuteMsg,
        account_address: &Addr,
    ) -> ExecuteMsg {
        AdapterRequestMsg {
            account_address: Some(account_address.to_string()),
            request,
        }
        .into()
    }

    // Only raw actions tested here, since it's pointless to resolve ans query in Mock tests

    #[test]
    fn swap_msg() {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let account = test_account(deps.api);
        deps.querier = abstract_adapter::abstract_testing::abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let dex = stub
            .dex(deps.as_ref(), &env, "junoswap".into())
            .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);
        let abstr = AbstractMockAddrs::new(deps.api);

        let dex_name = "junoswap".to_string();
        let offer_asset = Asset::native("ujuno", 100_000u128);
        let ask_asset = AssetInfo::native("uusd");
        let max_spread = Some(Decimal::percent(1));
        let belief_price = Some(Decimal::percent(2));
        let pool = PoolAddressBase::Id(POOL);

        let expected = expected_request_with_test_account(
            DexExecuteMsg::Action {
                dex: dex_name,
                action: DexAction::Swap {
                    offer_asset: offer_asset.clone().into(),
                    ask_asset: ask_asset.clone().into(),
                    max_spread,
                    belief_price,
                    pool: pool.clone().into(),
                },
            },
            account.addr(),
        );

        let actual = dex.swap(offer_asset, ask_asset, max_spread, belief_price, pool);

        assert!(actual.is_ok());

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn provide_liquidity_msg() {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let account = test_account(deps.api);
        deps.querier = abstract_adapter::abstract_testing::abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let dex_name = "junoswap".to_string();
        let abstr = AbstractMockAddrs::new(deps.api);

        let dex = stub
            .dex(deps.as_ref(), &env, dex_name.clone())
            .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

        let assets = vec![Asset::native("taco", 1000u128)];
        let max_spread = Some(Decimal::percent(1));
        let pool = PoolAddressBase::Id(POOL);

        let expected = expected_request_with_test_account(
            DexExecuteMsg::Action {
                dex: dex_name,
                action: DexAction::ProvideLiquidity {
                    assets: assets.clone().into_iter().map(Into::into).collect(),
                    max_spread,
                    pool: pool.clone().into(),
                },
            },
            account.addr(),
        );

        let actual = dex.provide_liquidity(assets, max_spread, pool);

        assert!(actual.is_ok());

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn withdraw_liquidity_msg() {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let account = test_account(deps.api);
        deps.querier = abstract_adapter::abstract_testing::abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let stub = MockModule::new(deps.api, account.clone());
        let dex_name = "junoswap".to_string();
        let abstr = AbstractMockAddrs::new(deps.api);

        let dex = stub
            .dex(deps.as_ref(), &env, dex_name.clone())
            .with_module_id(abstract_adapter::abstract_testing::prelude::TEST_MODULE_ID);

        let lp_token = Asset::native("taco", 1000u128);
        let pool = PoolAddressBase::Id(POOL);

        let expected = expected_request_with_test_account(
            DexExecuteMsg::Action {
                dex: dex_name,
                action: DexAction::WithdrawLiquidity {
                    lp_token: lp_token.clone().into(),
                    pool: pool.clone().into(),
                },
            },
            account.addr(),
        );

        let actual = dex.withdraw_liquidity(lp_token, pool);

        assert!(actual.is_ok());

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(&abstr.module_address, &expected, vec![]).unwrap();

        assert_eq!(actual, expected);
    }
}
