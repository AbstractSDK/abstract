pub mod adapter;
pub mod api;
pub mod contract;
pub(crate) mod handlers;
mod oracles;
pub mod state {
    pub use abstract_oracle_standard::state::*;
}
pub mod msg {
    pub use abstract_oracle_standard::msg::*;
}
pub use abstract_oracle_standard::ORACLE_ADAPTER_ID;

pub use abstract_oracle_standard::OracleError;

// TODO:
// Export interface for use in SDK modules
// pub use crate::api::OracleInterface;

#[cfg(feature = "interface")]
pub mod interface {
    use crate::{contract::ORACLE_ADAPTER, msg::*};
    use abstract_interface::{AdapterDeployer, RegisteredModule};
    use abstract_sdk::features::ModuleIdentification;
    use cosmwasm_std::Empty;
    use cw_orch::{build::BuildPostfix, interface};
    use cw_orch::{contract::Contract, prelude::*};

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct OracleAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, OracleInstantiateMsg> for OracleAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for OracleAdapter<Chain> {
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
                    "abstract_oracle_adapter",
                    BuildPostfix::<Chain>::ChainName(self.get_chain()),
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
    }

    impl<Chain: CwEnv> From<Contract<Chain>> for OracleAdapter<Chain> {
        fn from(contract: Contract<Chain>) -> Self {
            Self(contract)
        }
    }

    impl<Chain: cw_orch::environment::CwEnv> abstract_interface::DependencyCreation
        for OracleAdapter<Chain>
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
