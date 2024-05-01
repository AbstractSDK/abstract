pub mod adapter;
pub mod api;
pub mod contract;
pub(crate) mod handlers;
mod platform_resolver;
pub mod state;
pub mod msg {
    pub use abstract_money_market_standard::msg::*;
}
#[cfg(feature = "testing")]
pub mod tester;
pub use abstract_money_market_standard::MONEY_MARKET_ADAPTER_ID;

// Export interface for use in SDK modules
pub use crate::api::MoneyMarketInterface;

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use cw_orch::prelude::*;

    pub use crate::contract::interface::MoneyMarketAdapter;

    impl<Chain: CwEnv> MoneyMarketAdapter<Chain> {
        // TODO
    }

    impl<Chain: cw_orch::environment::CwEnv>
        abstract_adapter::abstract_interface::DependencyCreation for MoneyMarketAdapter<Chain>
    {
        type DependenciesConfig = cosmwasm_std::Empty;

        fn dependency_install_configs(
            _configuration: Self::DependenciesConfig,
        ) -> Result<
            Vec<abstract_adapter::std::manager::ModuleInstallConfig>,
            abstract_adapter::abstract_interface::AbstractInterfaceError,
        > {
            Ok(vec![])
        }
    }
}
