//! Mock module for API and feature testing

use abstract_core::objects::dependency::StaticDependency;
use abstract_testing::prelude::*;
use cosmwasm_std::{Addr, Attribute, Binary, Deps, Event};

use crate::core::objects::module::ModuleId;
use crate::features::{
    AbstractNameService, AbstractRegistryAccess, AccountIdentification, CustomData, CustomEvents,
    Dependencies, Executables, ExecutionStack, ModuleEndpointResponse, ModuleIdentification,
};
use crate::AbstractSdkResult;
use abstract_core::objects::ans_host::AnsHost;
use abstract_core::objects::version_control::VersionControlContract;

// We implement the following traits here for the mock module (in this package) to avoid a circular dependency

impl ExecutionStack for MockModule {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.response.executables
    }
}

impl CustomEvents for MockModule {
    fn add_event<A: Into<Attribute>>(
        &mut self,
        event_name: &str,
        attributes: impl IntoIterator<Item = A>,
    ) {
        self.response.add_event(event_name, attributes)
    }

    fn events(&self) -> Vec<Event> {
        self.response.events()
    }

    fn add_attributes<A: Into<Attribute>>(&mut self, attributes: impl IntoIterator<Item = A>) {
        self.response.add_attributes(attributes)
    }

    fn attributes(&self) -> Vec<Attribute> {
        self.response.attributes()
    }
}

impl CustomData for MockModule {
    fn data(&self) -> Option<Binary> {
        self.response.data()
    }

    fn set_data(&mut self, data: impl Into<Binary>) {
        self.response.set_data(data)
    }
}

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
pub struct MockModule {
    /// The Cosmwasm response gets created inside the module structure
    pub response: ModuleEndpointResponse,
}

impl MockModule {
    /// mock constructor
    pub fn new() -> Self {
        Self::default()
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
