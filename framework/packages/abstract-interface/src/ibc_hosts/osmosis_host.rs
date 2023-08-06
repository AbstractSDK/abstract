use abstract_core::ibc_host::*;
use cosmwasm_std::Empty;

use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, Empty, QueryMsg, MigrateMsg)]
pub struct OsmosisHost<Chain>;

impl<Chain: CwEnv> OsmosisHost<Chain> {}

use cw_orch::{
    prelude::ArtifactsDir,
    prelude::{ContractWrapper, Mock, TxHandler},
};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl<Chain: CwEnv> Uploadable for IbcClient<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::ans_host::contract::execute,
                ::ans_host::contract::instantiate,
                ::ans_host::contract::query,
            )
            .with_migrate(::ans_host::contract::migrate),
        )
    }
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ibc_client")
            .unwrap()
    }
}
