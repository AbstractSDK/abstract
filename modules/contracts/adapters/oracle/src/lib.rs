pub mod api;
pub mod contract;
pub(crate) mod handlers;
pub mod oracles;
pub mod state;
pub mod msg {
    pub use abstract_oracle_standard::msg::*;
}
pub use abstract_oracle_standard::ORACLE_ADAPTER_ID;

// Export interface for use in SDK modules
pub use crate::api::OracleInterface;

#[cfg(feature = "testing")]
pub mod oracle_tester;

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use crate::{contract::ORACLE_ADAPTER, msg::*};
    use abstract_adapter::abstract_interface::{AdapterDeployer, RegisteredModule};
    use abstract_adapter::objects::dependency::StaticDependency;
    use abstract_adapter::sdk::features::ModuleIdentification;

    use abstract_adapter::traits::Dependencies;
    use abstract_oracle_standard::ORACLE_ADAPTER_ID;
    use cw_orch::{build::BuildPostfix, interface};
    use cw_orch::{contract::Contract, prelude::*};

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty, id=ORACLE_ADAPTER_ID)]
    pub struct OracleAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, Empty> for OracleAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for OracleAdapter<Chain> {
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
                    "abstract_oracle_adapter",
                    BuildPostfix::ChainName(chain),
                )
                .unwrap()
        }
    }

    impl<Chain: CwEnv> RegisteredModule for OracleAdapter<Chain> {
        type InitMsg = Empty;

        fn module_id<'a>() -> &'a str {
            ORACLE_ADAPTER.module_id()
        }

        fn module_version<'a>() -> &'a str {
            ORACLE_ADAPTER.version()
        }

        fn dependencies<'a>() -> &'a [StaticDependency] {
            ORACLE_ADAPTER.dependencies()
        }
    }

    impl<Chain: CwEnv> From<Contract<Chain>> for OracleAdapter<Chain> {
        fn from(contract: Contract<Chain>) -> Self {
            Self(contract)
        }
    }

    impl<Chain: cw_orch::environment::CwEnv>
        abstract_adapter::abstract_interface::DependencyCreation for OracleAdapter<Chain>
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
