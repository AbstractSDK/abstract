pub mod adapter;
pub mod api;
pub mod contract;
mod exchanges;
pub(crate) mod handlers;
pub mod state;

// Export interface for use in SDK modules
pub use crate::api::DexInterface;
//:{Dex, DexInterface};
pub const EXCHANGE: &str = "abstract:dex";

pub use abstract_dex_adapter_traits::msg;

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_exchange {
    pub use abstract_osmosis_adapter::dex::Osmosis;
}

#[cfg(feature = "interface")]
pub mod interface {
    use std::env;

    use crate::{msg::*, EXCHANGE};
    use abstract_core::{
        adapter::{self},
        objects::{AnsAsset, AssetEntry},
        MANAGER,
    };
    use abstract_interface::AbstractInterfaceError;
    use abstract_interface::AdapterDeployer;
    use abstract_interface::Manager;
    use cosmwasm_std::{Decimal, Empty, Uint128};
    use cw_orch::interface;
    use cw_orch::prelude::*;

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct DexAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, DexInstantiateMsg> for DexAdapter<Chain> {}

    pub const DEX_ADAPTER_CHAIN_NAME_VAR: &str = "__INTERNAL__DEPLOY_CHAIN_NAME";

    impl<Chain: CwEnv> Uploadable for DexAdapter<Chain> {
        fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ))
        }
        fn wasm(&self) -> WasmPath {
            let chain_name = env::var(DEX_ADAPTER_CHAIN_NAME_VAR).unwrap_or_else(|_| panic!("Internal env variable {}, needs to be set to use the abstract dex adapter", DEX_ADAPTER_CHAIN_NAME_VAR));
            artifacts_dir_from_workspace!()
                .find_wasm_path(format!("abstract_dex_adapter-{}", chain_name).as_str())
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
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let asset = AssetEntry::new(offer_asset.0);
            let ask_asset = AssetEntry::new(ask_asset);

            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: DexExecuteMsg::Action {
                    dex,
                    action: DexAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
                        ask_asset,
                        max_spread: Some(Decimal::percent(30)),
                        belief_price: None,
                    },
                },
            });
            manager.execute_on_module(EXCHANGE, swap_msg)?;
            Ok(())
        }

        /// Provide liquidity using Abstract's OS (registered in daemon_state).
        pub fn provide(
            &self,
            asset1: (&str, u128),
            asset2: (&str, u128),
            dex: String,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let asset_entry1 = AssetEntry::new(asset1.0);
            let asset_entry2 = AssetEntry::new(asset2.0);

            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: DexExecuteMsg::Action {
                    dex,
                    action: DexAction::ProvideLiquidity {
                        assets: vec![
                            AnsAsset::new(asset_entry1, asset1.1),
                            AnsAsset::new(asset_entry2, asset2.1),
                        ],
                        max_spread: Some(Decimal::percent(30)),
                    },
                },
            });
            manager.execute_on_module(EXCHANGE, swap_msg)?;
            Ok(())
        }

        /// Withdraw liquidity using Abstract's OS (registered in daemon_state).
        pub fn withdraw(
            &self,
            lp_token: &str,
            amount: impl Into<Uint128>,
            dex: String,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let lp_token = AssetEntry::new(lp_token);

            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: DexExecuteMsg::Action {
                    dex,
                    action: DexAction::WithdrawLiquidity {
                        lp_token,
                        amount: amount.into(),
                    },
                },
            });
            manager.execute_on_module(EXCHANGE, swap_msg)?;
            Ok(())
        }
    }
}
