//! Mock module for API and feature testing

use abstract_core::objects::dependency::{Dependency, StaticDependency};
use abstract_testing::prelude::*;
use cosmwasm_std::{Addr, Attribute, Binary, Deps, Event};

use crate::core::objects::module::ModuleId;
use crate::features::{
    AbstractNameService, AbstractRegistryAccess, AccountIdentification, CustomData, CustomEvents,
    Dependencies, DepsAccess, DepsType, Executables, ExecutionStack, ModuleEndpointResponse,
    ModuleIdentification,
};
use crate::AbstractSdkResult;
use abstract_core::objects::ans_host::AnsHost;
use abstract_core::objects::version_control::VersionControlContract;

impl<'m> DepsAccess for MockModule<'m> {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> cosmwasm_std::DepsMut<'b> {
        self.deps.deps_mut()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> Deps<'b> {
        self.deps.deps()
    }

    fn env(&self) -> cosmwasm_std::Env {
        self.deps.env()
    }

    fn message_info(&self) -> cosmwasm_std::MessageInfo {
        self.deps.message_info()
    }
}

impl<'m> ExecutionStack for MockModule<'m> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.response.executables
    }
}

impl<'m> CustomEvents for MockModule<'m> {
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

impl<'m> CustomData for MockModule<'m> {
    fn data(&self) -> Option<Binary> {
        self.response.data()
    }

    fn set_data(&mut self, data: impl Into<Binary>) {
        self.response.set_data(data)
    }
}

// We implement the following traits here for the mock module (in this package) to avoid a circular dependency
impl<'m> AccountIdentification for MockModule<'m> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(Addr::unchecked(TEST_PROXY))
    }
}

impl<'m> ModuleIdentification for MockModule<'m> {
    fn module_id(&self) -> &str {
        "mock_module"
    }
}

impl<'m> AbstractNameService for MockModule<'m> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        Ok(AnsHost {
            address: Addr::unchecked("ans"),
        })
    }
}

impl<'m> AbstractRegistryAccess for MockModule<'m> {
    fn abstract_registry(&self) -> AbstractSdkResult<VersionControlContract> {
        Ok(VersionControlContract {
            address: Addr::unchecked("abstract_registry"),
        })
    }
}

impl<'m> Dependencies for MockModule<'m> {
    fn dependencies(&self) -> Vec<Dependency> {
        vec![(&TEST_MODULE_DEP).into()]
    }
}

/// Dependency on the mock module
pub const TEST_MODULE_DEP: StaticDependency = StaticDependency::new(TEST_MODULE_ID, &[">1.0.0"]);
/// Nonexistent module
pub const FAKE_MODULE_ID: ModuleId = "fake_module";

/// A mock module that can be used for testing.
/// Identifies itself as [`TEST_MODULE_ID`].
pub struct MockModule<'a> {
    /// Cosmwasm imports are registered directly inside the module structure
    pub deps: DepsType<'a>,
    /// The Cosmwasm response gets created inside the module structure
    pub response: ModuleEndpointResponse,
}

impl<'a> MockModule<'a> {
    /// mock constructor
    pub fn new(deps: DepsType<'a>) -> Self {
        Self {
            deps,
            response: ModuleEndpointResponse::default(),
        }
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
