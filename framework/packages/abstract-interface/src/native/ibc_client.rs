use abstract_std::{
    ibc_client::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    IBC_CLIENT,
};

use cw_orch::{contract::Contract, interface, prelude::*};

use crate::RegisteredModule;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl<Chain: CwEnv> cw_blob::interface::DeterministicInstantiation<Chain> for IbcClient<Chain> {}

impl<Chain: CwEnv> Uploadable for IbcClient<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::ibc_client::contract::execute,
                ::ibc_client::contract::instantiate,
                ::ibc_client::contract::query,
            )
            .with_migrate(::ibc_client::contract::migrate)
            .with_reply(::ibc_client::contract::reply)
            .with_sudo(::ibc_client::contract::sudo),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("ibc_client")
            .unwrap()
    }
}

impl<Chain: CwEnv> RegisteredModule for IbcClient<Chain> {
    type InitMsg = cosmwasm_std::Empty;

    fn module_id<'a>() -> &'a str {
        IBC_CLIENT
    }
    fn module_version<'a>() -> &'a str {
        ibc_client::contract::CONTRACT_VERSION
    }

    fn dependencies<'a>() -> &'a [abstract_std::objects::dependency::StaticDependency] {
        &[]
    }
}

impl<Chain: CwEnv> From<Contract<Chain>> for IbcClient<Chain> {
    fn from(value: Contract<Chain>) -> Self {
        IbcClient(value)
    }
}
