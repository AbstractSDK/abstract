pub mod map_tester;
pub(crate) mod querier;

pub use querier::{querier, wrap_querier};

pub const TEST_PROXY: &str = "proxy_address";
pub const TEST_MANAGER: &str = "manager_address";
pub const TEST_MODULE_ADDRESS: &str = "test_module_address";
pub const TEST_ANS_HOST: &str = "test_ans_host_address";
pub const TEST_VERSION_CONTROL: &str = "version_control_address";
pub const TEST_MODULE_ID: &str = "test_module_id";
pub const TEST_OS_ID: u32 = 0;
