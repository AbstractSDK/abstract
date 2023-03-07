use abstract_os::objects::dependency::StaticDependency;
use abstract_testing::prelude::{TEST_MODULE_ID, TEST_PROXY};
use cosmwasm_std::{Addr, Deps};

use crate::features::{AbstractNameService, Dependencies, Identification, ModuleIdentification};
use crate::AbstractSdkResult;
use abstract_os::objects::ans_host::AnsHost;
use os::objects::module::ModuleId;

// We implement the following traits here for the mock module (in this package) to avoid a circular dependency
impl Identification for MockModule {
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(Addr::unchecked(TEST_PROXY))
    }
}

impl ModuleIdentification for MockModule {
    fn module_id(&self) -> &'static str {
        "mock_module"
    }
}

impl AbstractNameService for MockModule {
    fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
        Ok(AnsHost {
            address: Addr::unchecked("ans"),
        })
    }
}

impl Dependencies for MockModule {
    fn dependencies(&self) -> &[StaticDependency] {
        &[TEST_MODULE_DEP]
    }
}

pub const TEST_MODULE_DEP: StaticDependency = StaticDependency::new(TEST_MODULE_ID, &[">1.0.0"]);
/// Nonexistent module
pub const FAKE_MODULE_ID: ModuleId = "fake_module";

/// A mock module that can be used for testing.
/// Identifies itself as [`TEST_MODULE_ID`].
pub struct MockModule {}

impl MockModule {
    pub const fn new() -> Self {
        Self {}
    }
}

#[cosmwasm_schema::cw_serde]
pub struct MockModuleExecuteMsg {}

#[cosmwasm_schema::cw_serde]
pub struct MockModuleQueryMsg {}

impl abstract_os::api::ApiExecuteMsg for MockModuleExecuteMsg {}

impl abstract_os::api::ApiQueryMsg for MockModuleQueryMsg {}

impl abstract_os::app::AppExecuteMsg for MockModuleExecuteMsg {}

impl abstract_os::app::AppQueryMsg for MockModuleQueryMsg {}
