use abstract_std::{
    ibc_client::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    IBC_CLIENT,
};

use cw_orch::{contract::Contract, interface, prelude::*};

use crate::RegisteredModule;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl<Chain: CwEnv> Uploadable for IbcClient<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::ibc_client::contract::execute,
                ::ibc_client::contract::instantiate,
                ::ibc_client::contract::query,
            )
            .with_migrate(::ibc_client::contract::migrate),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ibc_client")
            .unwrap()
    }
}

impl<Chain: CwEnv> RegisteredModule for IbcClient<Chain> {
    type InitMsg = InstantiateMsg;

    fn module_id<'a>() -> &'a str {
        IBC_CLIENT
    }
    fn module_version<'a>() -> &'a str {
        ibc_client::contract::CONTRACT_VERSION
    }
}

impl<Chain: CwEnv> From<Contract<Chain>> for IbcClient<Chain> {
    fn from(value: Contract<Chain>) -> Self {
        IbcClient(value)
    }
}
