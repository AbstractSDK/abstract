pub mod adapter;
pub mod api;
pub mod contract;
pub(crate) mod handlers;
mod platform_resolver;
pub mod state;
pub mod msg {
    pub use abstract_money_market_standard::msg::*;
}
#[cfg(feature = "test-utils")]
pub mod tester;
pub use abstract_money_market_standard::MONEY_MARKET_ADAPTER_ID;

// Export interface for use in SDK modules
pub use crate::api::MoneyMarketInterface;

#[cfg(feature = "interface")]
pub mod interface {
    use crate::{contract::MONEY_MARKET_ADAPTER, msg::*};
    use abstract_interface::{AdapterDeployer, RegisteredModule};
    use abstract_sdk::features::ModuleIdentification;
    use cosmwasm_std::Empty;
    use cw_orch::{build::BuildPostfix, interface};
    use cw_orch::{contract::Contract, prelude::*};

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct MoneyMarketAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, MoneyMarketInstantiateMsg> for MoneyMarketAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for MoneyMarketAdapter<Chain> {
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
                    "abstract_money_market_adapter",
                    BuildPostfix::<Chain>::ChainName(self.get_chain()),
                )
                .unwrap()
        }
    }

    impl<Chain: CwEnv> MoneyMarketAdapter<Chain> {
        // TODO
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

    impl<Chain: cw_orch::environment::CwEnv> abstract_interface::DependencyCreation
        for MoneyMarketAdapter<Chain>
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
