pub mod adapter;
pub mod api;
pub mod contract;
mod exchanges;
pub(crate) mod handlers;
pub mod state;
pub mod msg {
    pub use abstract_dex_standard::msg::*;
}
pub use abstract_dex_standard::DEX_ADAPTER_ID;

// Export interface for use in SDK modules
pub use crate::api::DexInterface;

#[cfg(any(feature = "wynd", feature = "osmosis"))]
pub mod host_exchange {
    pub use abstract_osmosis_adapter::dex::Osmosis;
}

#[cfg(feature = "testing")]
pub mod dex_tester;

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use crate::{contract::DEX_ADAPTER, msg::*};
    use abstract_adapter::abstract_interface::ClientResolve;
    use abstract_adapter::abstract_interface::{AbstractInterfaceError, AccountI, AnsHost};
    use abstract_adapter::abstract_interface::{AdapterDeployer, RegisteredModule};
    use abstract_adapter::objects::dependency::StaticDependency;
    use abstract_adapter::sdk::features::ModuleIdentification;
    use abstract_adapter::std::{
        adapter,
        objects::{pool_id::PoolAddressBase, AnsAsset, AssetEntry},
    };

    use abstract_adapter::traits::Dependencies;
    use abstract_dex_standard::ans_action::{DexAnsAction, WholeDexAction};
    use cosmwasm_std::Decimal;
    use cw_asset::{AssetBase, AssetInfoBase};
    use cw_orch::{build::BuildPostfix, interface};
    use cw_orch::{contract::Contract, prelude::*};

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct DexAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, DexInstantiateMsg> for DexAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for DexAdapter<Chain> {
        #[cfg(feature = "export")]
        fn wrapper() -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ))
        }
        fn wasm(chain: &ChainInfoOwned) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path_with_build_postfix(
                    "abstract_dex_adapter",
                    BuildPostfix::ChainName(chain),
                )
                .unwrap()
        }
    }

    impl<Chain: CwEnv> DexAdapter<Chain> {
        /// Ans action
        pub fn ans_action(
            &self,
            dex: String,
            action: DexAnsAction,
            account: impl AsRef<AccountI<Chain>>,
            ans_host: &AnsHost<Chain>,
        ) -> Result<<Chain as TxHandler>::Response, AbstractInterfaceError> {
            let account = account.as_ref();
            let request = WholeDexAction(dex, action).resolve(ans_host)?;
            let msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                account_address: Some(account.addr_str()?),
                request,
            });
            self.execute(&msg, &[]).map_err(Into::into)
        }

        /// Raw action
        pub fn raw_action(
            &self,
            dex: String,
            action: DexAction,
            account: impl AsRef<AccountI<Chain>>,
        ) -> Result<<Chain as TxHandler>::Response, AbstractInterfaceError> {
            let account = account.as_ref();
            let msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                account_address: Some(account.addr_str()?),
                request: DexExecuteMsg::Action { dex, action },
            });
            self.execute(&msg, &[]).map_err(Into::into)
        }

        /// Swap using ans resolved assets
        pub fn ans_swap(
            &self,
            offer_asset: (&str, u128),
            ask_asset: &str,
            dex: String,
            account: impl AsRef<AccountI<Chain>>,
            ans_host: &AnsHost<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let asset = AssetEntry::new(offer_asset.0);
            let ask_asset = AssetEntry::new(ask_asset);

            let action = DexAnsAction::Swap {
                offer_asset: AnsAsset::new(asset, offer_asset.1),
                ask_asset,
                max_spread: Some(Decimal::percent(30)),
                belief_price: None,
            };
            self.ans_action(dex, action, account, ans_host)?;
            Ok(())
        }

        /// Swap using raw native assets denoms
        pub fn raw_swap_native(
            &self,
            offer_asset: (&str, u128),
            ask_asset: &str,
            dex: String,
            account: impl AsRef<AccountI<Chain>>,
            pool: PoolAddressBase<String>,
        ) -> Result<(), AbstractInterfaceError> {
            let action = DexAction::Swap {
                offer_asset: AssetBase::native(offer_asset.0, offer_asset.1),
                ask_asset: AssetInfoBase::native(ask_asset),
                pool,
                max_spread: Some(Decimal::percent(30)),
                belief_price: None,
            };
            self.raw_action(dex, action, account)?;
            Ok(())
        }

        /// Provide liquidity using ans resolved assets
        pub fn ans_provide_liquidity(
            &self,
            assets: Vec<(&str, u128)>,
            dex: String,
            account: impl AsRef<AccountI<Chain>>,
            ans_host: &AnsHost<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let assets = assets.iter().map(|a| AnsAsset::new(a.0, a.1)).collect();

            let action = DexAnsAction::ProvideLiquidity {
                assets,
                max_spread: Some(Decimal::percent(30)),
            };
            self.ans_action(dex, action, account, ans_host)?;
            Ok(())
        }

        /// Provide Liquidity raw native assets denoms
        pub fn raw_provide_liquidity_native(
            &self,
            assets: Vec<(&str, u128)>,
            dex: String,
            account: impl AsRef<AccountI<Chain>>,
            pool: PoolAddressBase<String>,
        ) -> Result<(), AbstractInterfaceError> {
            let assets = assets.iter().map(|a| AssetBase::native(a.0, a.1)).collect();

            let action = DexAction::ProvideLiquidity {
                assets,
                pool,
                max_spread: Some(Decimal::percent(30)),
            };
            self.raw_action(dex, action, account)?;
            Ok(())
        }
    }

    impl<Chain: CwEnv> RegisteredModule for DexAdapter<Chain> {
        type InitMsg = Empty;

        fn module_id<'a>() -> &'a str {
            DEX_ADAPTER.module_id()
        }

        fn module_version<'a>() -> &'a str {
            DEX_ADAPTER.version()
        }

        fn dependencies<'a>() -> &'a [StaticDependency] {
            DEX_ADAPTER.dependencies()
        }
    }

    impl<Chain: CwEnv> From<Contract<Chain>> for DexAdapter<Chain> {
        fn from(contract: Contract<Chain>) -> Self {
            Self(contract)
        }
    }

    impl<Chain: cw_orch::environment::CwEnv>
        abstract_adapter::abstract_interface::DependencyCreation for DexAdapter<Chain>
    {
        type DependenciesConfig = cosmwasm_std::Empty;

        fn dependency_install_configs(
            _configuration: Self::DependenciesConfig,
        ) -> Result<
            Vec<abstract_adapter::std::account::ModuleInstallConfig>,
            abstract_adapter::abstract_interface::AbstractInterfaceError,
        > {
            Ok(vec![])
        }
    }
}
