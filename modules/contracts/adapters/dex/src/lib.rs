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

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_exchange {
    pub use abstract_osmosis_adapter::dex::Osmosis;
}

#[cfg(feature = "interface")]
pub mod interface {
    use crate::{contract::DEX_ADAPTER, msg::*, DEX_ADAPTER_ID};
    use abstract_core::{
        adapter::{self},
        objects::{AnsAsset, AssetEntry},
    };
    use abstract_interface::{AbstractAccount, AbstractInterfaceError};
    use abstract_interface::{AdapterDeployer, RegisteredModule};
    use abstract_sdk::base::Handler;
    use abstract_sdk::features::ModuleIdentification;
    use cosmwasm_std::{Decimal, Empty};
    use cw_orch::{build::BuildPostfix, interface};
    use cw_orch::{contract::Contract, prelude::*};

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct DexAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, DexInstantiateMsg> for DexAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for DexAdapter<Chain> {
        fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ))
        }
        fn wasm(&self) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path_with_build_postfix(
                    "abstract_dex_adapter",
                    BuildPostfix::<Chain>::ChainName(self.get_chain()),
                )
                .unwrap()
        }
    }

    impl<Chain: CwEnv> DexAdapter<Chain> {
        /// Swap using Abstract's OS (registered in daemon_state).
        pub fn swap(
            &self,
            offer_asset: (&str, u128),
            ask_asset: &str,
            dex: String,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let asset = AssetEntry::new(offer_asset.0);
            let ask_asset = AssetEntry::new(ask_asset);

            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: DexExecuteMsg::AnsAction {
                    dex,
                    action: DexAnsAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
                        ask_asset,
                        max_spread: Some(Decimal::percent(30)),
                        belief_price: None,
                    },
                },
            });
            account
                .manager
                .execute_on_module(DEX_ADAPTER_ID, swap_msg)?;
            Ok(())
        }
    }

    impl<Chain: CwEnv> RegisteredModule for DexAdapter<Chain> {
        type InitMsg = <crate::contract::DexAdapter as Handler>::CustomInitMsg;

        fn module_id<'a>() -> &'a str {
            DEX_ADAPTER.module_id()
        }

        fn module_version<'a>() -> &'a str {
            DEX_ADAPTER.version()
        }
    }

    impl<Chain: CwEnv> From<Contract<Chain>> for DexAdapter<Chain> {
        fn from(contract: Contract<Chain>) -> Self {
            Self(contract)
        }
    }

    impl<Chain: cw_orch::environment::CwEnv> abstract_interface::DependencyCreation
        for DexAdapter<Chain>
    {
        type DependenciesConfig = cosmwasm_std::Empty;

        fn dependency_install_configs(
            _configuration: Self::DependenciesConfig,
        ) -> Result<
            Vec<abstract_core::manager::ModuleInstallConfig>,
            abstract_interface::AbstractInterfaceError,
        > {
            Ok(vec![])
        }
    }
}
