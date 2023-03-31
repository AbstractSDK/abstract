pub(crate) mod abstract_mock_querier;
pub mod map_tester;
pub mod mock_ans;
pub(crate) mod mock_querier;
use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    OwnedDeps,
};
pub use mock_ans::MockAnsHost;
pub use mock_querier::{map_key, mock_querier, raw_map_key, wrap_querier, MockQuerierBuilder};
pub type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;
pub const OWNER: &str = "owner";
pub mod addresses {
    use abstract_core::version_control::AccountBase;
    use cosmwasm_std::Addr;

    pub const TEST_ADMIN: &str = "admin";
    pub const TEST_ACCOUNT_ID: u32 = 0;
    pub const TEST_VERSION: &str = "1.0.0";
    pub const TEST_PROXY: &str = "proxy_address";
    pub const TEST_MANAGER: &str = "manager_address";
    pub const TEST_ANS_HOST: &str = "test_ans_host_address";
    pub const TEST_VERSION_CONTROL: &str = "version_control_address";
    pub const TEST_ACCOUNT_FACTORY: &str = "account_factory_address";
    pub const TEST_MODULE_FACTORY: &str = "module_factory_address";
    pub const TEST_MODULE_ADDRESS: &str = "test_module_address";
    pub const TEST_MODULE_ID: &str = "tester:test-module-id";

    pub const TEST_MODULE_RESPONSE: &str = "test_module_response";

    pub const TEST_DEX: &str = "test_dex";
    pub const TTOKEN: &str = "test_token";
    pub const EUR_USD_PAIR: &str = "dex:eur_usd_pair";
    pub const EUR_USD_LP: &str = "dex/eur,usd";
    pub const TTOKEN_EUR_PAIR: &str = "dex:wynd_eur_pair";
    pub const TTOKEN_EUR_LP: &str = "dex/wynd,eur";
    pub const EUR: &str = "eur";
    pub const USD: &str = "usd";

    /// TODO: static const?
    pub fn test_core() -> AccountBase {
        AccountBase {
            manager: Addr::unchecked(TEST_MANAGER),
            proxy: Addr::unchecked(TEST_PROXY),
        }
    }
}

pub mod prelude {
    use super::*;

    pub use super::OWNER;
    pub use abstract_mock_querier::{mocked_account_querier_builder, AbstractMockQuerierBuilder};
    pub use addresses::*;
    pub use mock_querier::{map_key, mock_querier, raw_map_key, wrap_querier, MockQuerierBuilder};

    pub use super::MockDeps;

    pub use cosmwasm_std::{
        from_binary,
        testing::{MockApi as CwMockApi, MockQuerier, MockStorage},
        to_binary,
    };
}
