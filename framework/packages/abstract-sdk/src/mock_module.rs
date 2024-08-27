//! Mock module for API and feature testing

use abstract_std::objects::{
    ans_host::AnsHost, dependency::StaticDependency, version_control::VersionControlContract,
};
use abstract_testing::prelude::*;
use cosmwasm_std::{testing::MockApi, Addr, Deps};

use crate::{
    features::{
        AbstractNameService, AbstractRegistryAccess, AccountExecutor, AccountIdentification,
        Dependencies, ModuleIdentification,
    },
    std::objects::module::ModuleId,
    AbstractSdkResult,
};

// We implement the following traits here for the mock module (in this package) to avoid a circular dependency
impl AccountIdentification for MockModule {
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        let test_base = test_account_base(self.mock_api);
        Ok(test_base.proxy)
    }
}

impl AccountExecutor for MockModule {}

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

impl AbstractRegistryAccess for MockModule {
    fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<VersionControlContract> {
        Ok(VersionControlContract {
            address: Addr::unchecked("abstract_registry"),
        })
    }
}

impl Dependencies for MockModule {
    fn dependencies(&self) -> &'static [StaticDependency] {
        &[TEST_MODULE_DEP]
    }
}

/// Dependency on the mock module
pub const TEST_MODULE_DEP: StaticDependency = StaticDependency::new(TEST_MODULE_ID, &[">1.0.0"]);
/// Nonexistent module
pub const FAKE_MODULE_ID: ModuleId = "fake_module";

/// A mock module that can be used for testing.
/// Identifies itself as [`TEST_MODULE_ID`].
pub struct MockModule {
    mock_api: MockApi,
}

impl MockModule {
    /// mock constructor
    pub fn new(mock_api: MockApi) -> Self {
        Self { mock_api }
    }
}

/// Mock module execute message
#[cosmwasm_schema::cw_serde]
pub struct MockModuleExecuteMsg {}

/// Mock module query message
#[cosmwasm_schema::cw_serde]
pub struct MockModuleQueryMsg {}

/// Mock module query message
#[cosmwasm_schema::cw_serde]
pub struct MockModuleQueryResponse {}

impl abstract_std::adapter::AdapterExecuteMsg for MockModuleExecuteMsg {}

impl abstract_std::adapter::AdapterQueryMsg for MockModuleQueryMsg {}

impl abstract_std::app::AppExecuteMsg for MockModuleExecuteMsg {}

impl abstract_std::app::AppQueryMsg for MockModuleQueryMsg {}
