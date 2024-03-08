pub mod adapter;
pub mod api;
pub mod contract;
mod exchanges;
pub(crate) mod handlers;
pub mod state;
pub mod msg {
    pub use abstract_moneymarket_standard::msg::*;
}
pub use abstract_moneymarket_standard::MONEYMARKET_ADAPTER_ID;

// Export interface for use in SDK modules
pub use crate::api::MoneymarketInterface;

#[cfg(feature = "interface")]
pub mod interface {
    use crate::{contract::MONEYMARKET_ADAPTER, msg::*, MONEYMARKET_ADAPTER_ID};
    use abstract_core::{
        adapter,
        objects::{pool_id::PoolAddressBase, AnsAsset, AssetEntry},
    };
    use abstract_interface::{AbstractAccount, AbstractInterfaceError};
    use abstract_interface::{AdapterDeployer, RegisteredModule};
    use abstract_moneymarket_standard::ans_action::MoneymarketAnsAction;
    use abstract_moneymarket_standard::raw_action::MoneymarketRawAction;
    use abstract_sdk::base::Handler;
    use abstract_sdk::features::ModuleIdentification;
    use cosmwasm_std::{Decimal, Empty};
    use cw_asset::{AssetBase, AssetInfoBase};
    use cw_orch::{build::BuildPostfix, interface};
    use cw_orch::{contract::Contract, prelude::*};

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct MoneymarketAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, MoneymarketInstantiateMsg> for MoneymarketAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for MoneymarketAdapter<Chain> {
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
                    "abstract_moneymarket_adapter",
                    BuildPostfix::<Chain>::ChainName(self.get_chain()),
                )
                .unwrap()
        }
    }

    impl<Chain: CwEnv> MoneymarketAdapter<Chain> {
        /// Swap using ans resolved assets
        pub fn ans_swap(
            &self,
            offer_asset: (&str, u128),
            ask_asset: &str,
            moneymarket: String,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let asset = AssetEntry::new(offer_asset.0);
            let ask_asset = AssetEntry::new(ask_asset);

            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: MoneymarketExecuteMsg::AnsAction {
                    moneymarket,
                    action: MoneymarketAnsAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
                        ask_asset,
                        max_spread: Some(Decimal::percent(30)),
                        belief_price: None,
                    },
                },
            });
            account
                .manager
                .execute_on_module(MONEYMARKET_ADAPTER_ID, swap_msg)?;
            Ok(())
        }
        /// Swap using raw asset addresses
        pub fn raw_swap_native(
            &self,
            offer_asset: (&str, u128),
            ask_asset: &str,
            moneymarket: String,
            account: &AbstractAccount<Chain>,
            pool: PoolAddressBase<String>,
        ) -> Result<(), AbstractInterfaceError> {
            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: MoneymarketExecuteMsg::RawAction {
                    moneymarket,
                    action: MoneymarketRawAction::Swap {
                        offer_asset: AssetBase::native(offer_asset.0, offer_asset.1),
                        ask_asset: AssetInfoBase::native(ask_asset),
                        pool,
                        max_spread: Some(Decimal::percent(30)),
                        belief_price: None,
                    },
                },
            });
            account
                .manager
                .execute_on_module(MONEYMARKET_ADAPTER_ID, swap_msg)?;
            Ok(())
        }
    }

    impl<Chain: CwEnv> RegisteredModule for MoneymarketAdapter<Chain> {
        type InitMsg = <crate::contract::MoneymarketAdapter as Handler>::CustomInitMsg;

        fn module_id<'a>() -> &'a str {
            MONEYMARKET_ADAPTER.module_id()
        }

        fn module_version<'a>() -> &'a str {
            MONEYMARKET_ADAPTER.version()
        }
    }

    impl<Chain: CwEnv> From<Contract<Chain>> for MoneymarketAdapter<Chain> {
        fn from(contract: Contract<Chain>) -> Self {
            Self(contract)
        }
    }

    impl<Chain: cw_orch::environment::CwEnv> abstract_interface::DependencyCreation
        for MoneymarketAdapter<Chain>
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
