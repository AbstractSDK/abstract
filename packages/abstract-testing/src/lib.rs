pub mod map_tester;
pub mod mock_module;
pub(crate) mod mock_querier;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::OwnedDeps;
pub use mock_querier::{querier, wrap_querier};

pub type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

pub const TEST_PROXY: &str = "proxy_address";
pub const TEST_MANAGER: &str = "manager_address";
pub const TEST_ANS_HOST: &str = "test_ans_host_address";
pub const TEST_VERSION_CONTROL: &str = "version_control_address";
pub const TEST_ADMIN: &str = "admin";
pub const TEST_MODULE_ADDRESS: &str = "test_module_address";
pub const TEST_MODULE_ID: &str = "test_module_id";
pub const TEST_OS_ID: u32 = 0;
pub const TEST_VERSION: &str = "1.0.0";
pub const TEST_MODULE_RESPONSE: &str = "test_module_response";
