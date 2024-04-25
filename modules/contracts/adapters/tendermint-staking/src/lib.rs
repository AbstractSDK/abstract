pub mod contract;
pub mod error;
pub mod msg;
mod staking;

pub const TENDERMINT_STAKING: &str = "abstract:tendermint-staking";

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {

    use abstract_interface::AdapterDeployer;
    use cosmwasm_std::Empty;
    use cw_orch::{
        environment::CwEnv,
        interface,
        prelude::{ContractWrapper, *},
    };

    use crate::msg::*;

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct TMintStakingAdapter;

    impl<Chain: CwEnv> AdapterDeployer<Chain, Empty> for TMintStakingAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for TMintStakingAdapter<Chain> {
        fn wrapper() -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ))
        }
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path("abstract_tendermint_staking_adapter")
                .unwrap()
        }
    }
}
