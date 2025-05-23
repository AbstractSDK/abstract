//! Mock module for API and feature testing

use abstract_std::{
    objects::{ans_host::AnsHost, dependency::StaticDependency, registry::RegistryContract},
    registry::Account,
};
use abstract_testing::prelude::*;
use cosmwasm_std::{
    testing::{mock_dependencies, MockApi},
    Deps,
};

use crate::{
    features::{
        AbstractNameService, AbstractRegistryAccess, AccountIdentification, Dependencies,
        ModuleIdentification,
    },
    std::objects::module::ModuleId,
    AbstractSdkResult,
};

// We implement the following traits here for the mock module (in this package) to avoid a circular dependency
impl AccountIdentification for MockModule {
    fn account(&self, _deps: Deps) -> AbstractSdkResult<Account> {
        Ok(self.account.clone())
    }
}

impl ModuleIdentification for MockModule {
    fn module_id(&self) -> &'static str {
        TEST_MODULE_ID
    }
}

impl AbstractNameService for MockModule {
    fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
        let abstr = AbstractMockAddrs::new(self.mock_api);
        Ok(AnsHost {
            address: abstr.ans_host,
        })
    }
}

impl AbstractRegistryAccess for MockModule {
    fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<RegistryContract> {
        let abstr = AbstractMockAddrs::new(self.mock_api);
        Ok(RegistryContract {
            address: abstr.registry,
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
    account: Account,
}

impl MockModule {
    /// mock constructor
    pub fn new(mock_api: MockApi, account: Account) -> Self {
        Self { mock_api, account }
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

/// [`MockModule`] test setup
pub fn mock_module_setup() -> (MockDeps, Account, MockModule) {
    let mut deps = mock_dependencies();
    let account = test_account(deps.api);
    deps.querier = abstract_mock_querier_builder(deps.api)
        .account(&account, TEST_ACCOUNT_ID)
        .build();
    let app = MockModule::new(deps.api, account.clone());

    (deps, account, app)
}
