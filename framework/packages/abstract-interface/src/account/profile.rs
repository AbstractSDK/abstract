use abstract_std::{profile::*, PROFILE};
use bs_profile::Metadata;
use cw_orch::{interface, prelude::*};

// abstract interface
#[interface(InstantiateMsg, ExecuteMsg<T>, QueryMsg, MigrateMsg)]
pub struct Profile<Chain, T>;

impl<Chain: CwEnv> Uploadable for Profile<Chain, Metadata> {
    #[cfg(feature = "integration")]
    fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::profile::contract::execute,
                ::profile::contract::instantiate,
                ::profile::contract::query,
            )
            .with_migrate(::profile::contract::migrate), // .with_reply(::profile::contract::reply),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("bs721_profile")
            .unwrap()
    }
}
