pub(crate) mod abstract_mock_querier;
pub mod map_tester;
pub mod mock_module;
pub(crate) mod mock_querier;
use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    OwnedDeps,
};
pub use mock_querier::{map_key, mock_querier, raw_map_key, wrap_querier, MockQuerierBuilder};

pub type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

mod test_addresses {
    use abstract_os::version_control::Core;
    use cosmwasm_std::Addr;

    pub const TEST_ADMIN: &str = "admin";
    pub const TEST_OS_ID: u32 = 0;
    pub const TEST_VERSION: &str = "1.0.0";

    pub const TEST_PROXY: &str = "proxy_address";
    pub const TEST_MANAGER: &str = "manager_address";
    pub const TEST_ANS_HOST: &str = "test_ans_host_address";
    pub const TEST_VERSION_CONTROL: &str = "version_control_address";
    pub const TEST_OS_FACTORY: &str = "os_factory_address";
    pub const TEST_MODULE_FACTORY: &str = "module_factory_address";
    pub const TEST_MODULE_ADDRESS: &str = "test_module_address";
    pub const TEST_MODULE_ID: &str = "test-module-id";

    pub const TEST_MODULE_RESPONSE: &str = "test_module_response";

    /// TODO: static const?
    pub fn test_core() -> Core {
        Core {
            manager: Addr::unchecked(TEST_MANAGER),
            proxy: Addr::unchecked(TEST_PROXY),
        }
    }
}

pub use test_addresses::*;

pub mod prelude {
    use super::*;

    pub use abstract_mock_querier::AbstractMockQuerierBuilder;
    pub use mock_module::mocked_os_querier_builder;
    pub use mock_querier::{map_key, mock_querier, raw_map_key, wrap_querier, MockQuerierBuilder};
    pub use test_addresses::*;

    pub use super::MockDeps;

    pub use cosmwasm_std::{
        from_binary,
        testing::{MockApi, MockQuerier, MockStorage},
        to_binary,
    };

    pub use mock_module::*;
}
