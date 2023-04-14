pub mod contract;
pub mod error;
mod handlers;
pub mod msg;
pub mod response;
pub mod state;

pub const ETF_ID: &str = "abstract:etf";

#[cfg(feature = "boot")]
pub mod boot {
    use crate::msg::*;
    use abstract_boot::AppDeployer;
    use abstract_core::app::MigrateMsg;
    use boot_core::ContractWrapper;
    use boot_core::{contract, Contract, CwEnv};

    #[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
    pub struct ETF<Chain>;

    impl<Chain: CwEnv> AppDeployer<Chain> for ETF<Chain> {}

    impl<Chain: CwEnv> ETF<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            let mut contract = Contract::new(name, chain);
            contract = contract.with_wasm_path("etf").with_mock(Box::new(
                ContractWrapper::new_with_empty(
                    crate::contract::execute,
                    crate::contract::instantiate,
                    crate::contract::query,
                )
                .with_reply(crate::contract::reply),
            ));
            Self(contract)
        }
    }
}
