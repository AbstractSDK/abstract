//! Mock module for API and feature testing

use std::marker::PhantomData;

use abstract_core::objects::dependency::{Dependency, StaticDependency};
use abstract_testing::prelude::{TEST_MODULE_ID, TEST_PROXY};
use cosmwasm_std::{Addr, Attribute, Binary, Deps, Event};

use crate::core::objects::module::ModuleId;
use crate::features::{
    AbstractNameService, AbstractRegistryAccess, AccountIdentification, CustomData, CustomEvents,
    Dependencies, DepsAccess, Executables, ExecutionStack, HasExecutableEnv, ModuleIdentification,
};
use crate::AbstractSdkResult;
use abstract_core::objects::ans_host::AnsHost;
use abstract_core::objects::version_control::VersionControlContract;

impl<'m, T: DepsAccess> DepsAccess for MockModule<'m, T> {
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

impl<'m, T: DepsAccess + HasExecutableEnv> ExecutionStack for MockModule<'m, T> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.executables
    }
}

impl<'m, T: DepsAccess + HasExecutableEnv> CustomEvents for MockModule<'m, T> {
    fn add_event<A: Into<Attribute>>(
        &mut self,
        event_name: &str,
        attributes: impl IntoIterator<Item = A>,
    ) {
        let event = Event::new(event_name).add_attributes(attributes);
        self.events.push(event)
    }

    fn events(&self) -> Vec<Event> {
        self.events.clone()
    }

    fn add_attributes<A: Into<Attribute>>(&mut self, attributes: impl IntoIterator<Item = A>) {
        self.attributes
            .extend(attributes.into_iter().map(Into::into))
    }

    fn attributes(&self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

impl<'m, T: DepsAccess + HasExecutableEnv> CustomData for MockModule<'m, T> {
    fn data(&self) -> Option<Binary> {
        self.data.clone()
    }

    fn set_data(&mut self, data: impl Into<Binary>) {
        self.data = Some(data.into())
    }
}

// We implement the following traits here for the mock module (in this package) to avoid a circular dependency
impl<'m, T: DepsAccess> AccountIdentification for MockModule<'m, T> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(Addr::unchecked(TEST_PROXY))
    }
}

impl<'m, T: DepsAccess> ModuleIdentification for MockModule<'m, T> {
    fn module_id(&self) -> String {
        "mock_module".to_string()
    }
}

impl<'m, T: DepsAccess> AbstractNameService for MockModule<'m, T> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        Ok(AnsHost {
            address: Addr::unchecked("ans"),
        })
    }
}

impl<'m, T: DepsAccess> AbstractRegistryAccess for MockModule<'m, T> {
    fn abstract_registry(&self) -> AbstractSdkResult<VersionControlContract> {
        Ok(VersionControlContract {
            address: Addr::unchecked("abstract_registry"),
        })
    }
}

impl<'m, T: DepsAccess> Dependencies for MockModule<'m, T> {
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
pub struct MockModule<'a, T: DepsAccess> {
    deps: T,
    pub executables: Executables,
    pub events: Vec<Event>,
    attributes: Vec<Attribute>,
    data: Option<Binary>,
    lifetime: PhantomData<&'a ()>,
}

impl<'a, T: DepsAccess> MockModule<'a, T> {
    /// mock constructor
    pub fn new(deps: T) -> Self {
        Self {
            deps,
            events: vec![],
            attributes: vec![],
            executables: Executables::default(),
            data: None,
            lifetime: PhantomData,
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
