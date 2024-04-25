pub use abstract_core::ibc_host::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcHost<Chain>;

impl<Chain: CwEnv> Uploadable for IbcHost<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ibc_host::contract::execute,
                ibc_host::contract::instantiate,
                ibc_host::contract::query,
            )
            .with_migrate(ibc_host::contract::migrate)
            .with_reply(ibc_host::contract::reply),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ibc_host")
            .unwrap()
    }
}
