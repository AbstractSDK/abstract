pub mod adapter;
pub mod api;
pub mod contract;
pub(crate) mod handlers;
mod platform_resolver;
pub mod state;
pub mod msg {
    pub use abstract_money_market_standard::msg::*;
}
#[cfg(feature = "testing")]
pub mod tester;
pub use abstract_money_market_standard::MONEY_MARKET_ADAPTER_ID;

// Export interface for use in SDK modules
pub use crate::api::MoneyMarketInterface;

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use abstract_adapter::{
        abstract_interface::{AdapterDeployer, RegisteredModule},
        traits::ModuleIdentification as _,
    };
    use abstract_money_market_standard::msg::{
        ExecuteMsg, InstantiateMsg, MoneyMarketInstantiateMsg, QueryMsg,
    };

    use cw_orch::{build::BuildPostfix, contract::Contract, prelude::*};

    use crate::contract::MONEY_MARKET_ADAPTER;

    #[cw_orch::interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct MoneyMarketAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, MoneyMarketInstantiateMsg> for MoneyMarketAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for MoneyMarketAdapter<Chain> {
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
                    "abstract_money_market_adapter",
                    BuildPostfix::TruncatedChainId(chain),
                )
                .unwrap()
        }
    }

    impl<Chain: cw_orch::environment::CwEnv>
        abstract_adapter::abstract_interface::DependencyCreation for MoneyMarketAdapter<Chain>
    {
        type DependenciesConfig = cosmwasm_std::Empty;

        fn dependency_install_configs(
            _configuration: Self::DependenciesConfig,
        ) -> Result<
            Vec<abstract_adapter::std::manager::ModuleInstallConfig>,
            abstract_adapter::abstract_interface::AbstractInterfaceError,
        > {
            Ok(vec![])
        }
    }

    impl<Chain: CwEnv> RegisteredModule for MoneyMarketAdapter<Chain> {
        type InitMsg = Empty;

        fn module_id<'a>() -> &'a str {
            MONEY_MARKET_ADAPTER.module_id()
        }

        fn module_version<'a>() -> &'a str {
            MONEY_MARKET_ADAPTER.version()
        }
    }

    impl<Chain: CwEnv> From<Contract<Chain>> for MoneyMarketAdapter<Chain> {
        fn from(contract: Contract<Chain>) -> Self {
            Self(contract)
        }
    }
}
