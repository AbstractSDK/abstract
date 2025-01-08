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

    pub mod deployment {
        use cosmwasm_std::Addr;
        use cw_orch::daemon::networks::{NEUTRON_1, OSMOSIS_1, OSMO_5, PION_1, XION_TESTNET_1};
        use std::collections::HashMap;

        pub fn pyth_addresses() -> HashMap<String, Addr> {
            vec![
                (XION_TESTNET_1.chain_id, PYTH_XION_TEST_ADDRESS),
                (PION_1.chain_id, PYTH_PION_ADDRESS),
                (OSMO_5.chain_id, PYTH_OSMO_TEST_ADDRESS),
                (NEUTRON_1.chain_id, PYTH_NEUTRON_ADDRESS),
                (OSMOSIS_1.chain_id, PYTH_OSMOSIS_ADDRESS),
            ]
            .into_iter()
            .map(|(key, value)| (key.to_string(), Addr::unchecked(value)))
            .collect()
        }
        Source: https://docs.pyth.network/price-feeds/contract-addresses/cosmwasm
        pub const PYTH_XION_TEST_ADDRESS: &str =
            "xion1w39ctwxxhxxc2kxarycjxj9rndn65gf8daek7ggarwh3rq3zl0lqqllnmt";
        pub const PYTH_PION_ADDRESS: &str =
            "neutron15ldst8t80982akgr8w8ekcytejzkmfpgdkeq4xgtge48qs7435jqp87u3t";
        pub const PYTH_OSMO_TEST_ADDRESS: &str =
            "osmo1hpdzqku55lmfmptpyj6wdlugqs5etr6teqf7r4yqjjrxjznjhtuqqu5kdh";

        pub const PYTH_NEUTRON_ADDRESS: &str =
            "neutron1m2emc93m9gpwgsrsf2vylv9xvgqh654630v7dfrhrkmr5slly53spg85wv";
        pub const PYTH_OSMOSIS_ADDRESS: &str =
            "osmo13ge29x4e2s63a8ytz2px8gurtyznmue4a69n5275692v3qn3ks8q7cwck7";
    }
}
