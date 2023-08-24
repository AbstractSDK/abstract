use cw_orch::{
    interface,
    prelude::{artifacts_dir_from_workspace, CwEnv, Uploadable, WasmPath},
};

use abstract_core::ibc_client::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
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
                ibc_client::contract::execute,
                ibc_client::contract::instantiate,
                ::ibc_client::contract::query,
            )
            .with_migrate(::ibc_client::contract::migrate),
        )
    }
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ibc_client")
            .unwrap()
    }
}