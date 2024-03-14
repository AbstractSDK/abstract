pub mod adapter;
pub mod api;
pub mod contract;
pub(crate) mod handlers;
mod platform_resolver;
pub mod state;
pub mod msg {
    pub use abstract_money_market_standard::msg::*;
}
pub use abstract_money_market_standard::MONEYMARKET_ADAPTER_ID;

// Export interface for use in SDK modules
pub use crate::api::MoneymarketInterface;

#[cfg(feature = "interface")]
pub mod interface {
    use crate::{contract::MONEYMARKET_ADAPTER, msg::*};
    use abstract_interface::{AdapterDeployer, RegisteredModule};
    use abstract_sdk::base::Handler;
    use abstract_sdk::features::ModuleIdentification;
    use cosmwasm_std::Empty;
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
                    "abstract_money_market_adapter",
                    BuildPostfix::<Chain>::ChainName(self.get_chain()),
                )
                .unwrap()
        }
    }

    impl<Chain: CwEnv> MoneymarketAdapter<Chain> {
        // TODO
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
