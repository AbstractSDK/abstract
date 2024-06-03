use abstract_std::profile_marketplace::*;
use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct ProfileMarketplace<Chain>;

impl<Chain: CwEnv> Uploadable for ProfileMarketplace<Chain> {
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::profile_marketplace::contract::execute,
                ::profile_marketplace::contract::instantiate,
                ::profile_marketplace::contract::query,
            )
            .with_migrate(::profile_marketplace::contract::migrate)
            .with_reply(::profile_marketplace::contract::reply),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("profile_marketplace")
            .unwrap()
    }
}
