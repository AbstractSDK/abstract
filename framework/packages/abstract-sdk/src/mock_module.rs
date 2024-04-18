//! Mock module for API and feature testing

use abstract_core::objects::{
    ans_host::AnsHost, dependency::StaticDependency, version_control::VersionControlContract,
};
use abstract_testing::prelude::*;
use cosmwasm_std::{Addr, Deps};

use crate::{
    core::objects::module::ModuleId,
    features::{
        AbstractNameService, AbstractRegistryAccess, AccountIdentification, Dependencies,
        ModuleIdentification,
    },
    AbstractSdkResult,
};

// We implement the following traits here for the mock module (in this package) to avoid a circular dependency
impl AccountIdentification for MockModule {
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

impl AbstractRegistryAccess for MockModule {
    fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<VersionControlContract> {
        Ok(VersionControlContract {
            address: Addr::unchecked("abstract_registry"),
        })
    }
}

impl Dependencies for MockModule {
    fn dependencies(&self) -> &[StaticDependency] {
        &[TEST_MODULE_DEP]
    }
}

/// Dependency on the mock module
pub const TEST_MODULE_DEP: StaticDependency = StaticDependency::new(TEST_MODULE_ID, &[">1.0.0"]);
/// Nonexistent module
pub const FAKE_MODULE_ID: ModuleId = "fake_module";

/// A mock module that can be used for testing.
/// Identifies itself as [`TEST_MODULE_ID`].
#[derive(Default)]
pub struct MockModule {}

impl MockModule {
    /// mock constructor
    pub const fn new() -> Self {
        Self {}
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

impl abstract_core::adapter::AdapterExecuteMsg for MockModuleExecuteMsg {}

impl abstract_core::adapter::AdapterQueryMsg for MockModuleQueryMsg {}

impl abstract_core::app::AppExecuteMsg for MockModuleExecuteMsg {}

impl abstract_core::app::AppQueryMsg for MockModuleQueryMsg {}
