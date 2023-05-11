pub mod contract;
pub mod error;
pub mod msg;
mod staking;

pub const TENDERMINT_STAKING: &str = "abstract:tendermint-staking";

#[cfg(feature = "boot")]
pub mod boot {
    use abstract_boot::{
        boot_core::ContractWrapper,
        boot_core::{contract, Contract, CwEnv},
        AdapterDeployer,
    };
    use abstract_core::adapter::InstantiateMsg;
    use cosmwasm_std::Empty;

    use crate::msg::*;

    #[contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct TMintStakingAdapter<Chain>;

    impl<Chain: CwEnv> AdapterDeployer<Chain, Empty> for TMintStakingAdapter<Chain> {}

    impl<Chain: CwEnv> TMintStakingAdapter<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            Self(
                Contract::new(name, chain)
                    .with_wasm_path("abstract_tendermint_staking_adapter")
                    .with_mock(Box::new(ContractWrapper::new_with_empty(
                        crate::contract::execute,
                        crate::contract::instantiate,
                        crate::contract::query,
                    ))),
            )
        }
    }
}
