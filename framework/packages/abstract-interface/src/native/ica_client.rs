use abstract_std::{
    ica_client::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    ICA_CLIENT,
};

use cw_orch::{contract::Contract, interface, prelude::*};

use crate::RegisteredModule;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IcaClient<Chain>;

impl<Chain: CwEnv> Uploadable for IcaClient<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::ica_client::contract::execute,
                ::ica_client::contract::instantiate,
                ::ica_client::contract::query,
            )
            .with_migrate(::ica_client::contract::migrate),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ica_client")
            .unwrap()
    }
}

impl<Chain: CwEnv> RegisteredModule for IcaClient<Chain> {
    type InitMsg = cosmwasm_std::Empty;

    fn module_id<'a>() -> &'a str {
        ICA_CLIENT
    }
    fn module_version<'a>() -> &'a str {
        ica_client::contract::CONTRACT_VERSION
    }

    fn dependencies<'a>() -> &'a [abstract_std::objects::dependency::StaticDependency] {
        &[]
    }
}

impl<Chain: CwEnv> From<Contract<Chain>> for IcaClient<Chain> {
    fn from(value: Contract<Chain>) -> Self {
        IcaClient(value)
    }
}
