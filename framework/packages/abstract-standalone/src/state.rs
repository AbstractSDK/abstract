use abstract_sdk::{
    base::{ModuleId, ModuleMetadata, VersionString},
    features::ModuleIdentification,
    namespaces::{ADMIN_NAMESPACE, BASE_STATE},
};
use abstract_std::{
    objects::{
        dependency::StaticDependency, module::ModuleInfo, ownership::nested_admin::NestedAdmin,
    },
    standalone::StandaloneState,
    AbstractResult,
};
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;

/// The state variables for our StandaloneContract.
pub struct StandaloneContract {
    pub admin: NestedAdmin,
    pub(crate) base_state: Item<StandaloneState>,
    /// Static info about the contract, used for migration
    pub(crate) info: (ModuleId, VersionString, ModuleMetadata),
    /// Modules that this contract depends on.
    pub(crate) dependencies: &'static [StaticDependency],
}

impl ModuleIdentification for StandaloneContract {
    fn module_id(&self) -> abstract_std::objects::module::ModuleId<'static> {
        self.info.0
    }
}

/// Constructor
impl StandaloneContract {
    pub const fn new(
        name: &'static str,
        version: &'static str,
        metadata: Option<&'static str>,
    ) -> Self {
        Self {
            admin: NestedAdmin::new(ADMIN_NAMESPACE),
            base_state: Item::new(BASE_STATE),
            info: (name, version, metadata),
            dependencies: &[],
        }
    }

    pub fn module_id(&self) -> &'static str {
        self.info.0
    }

    pub fn version(&self) -> &'static str {
        self.info.1
    }

    pub fn module_info(&self) -> AbstractResult<ModuleInfo> {
        ModuleInfo::from_id(self.module_id(), self.version().into())
    }

    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [StaticDependency]) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn load_state(&self, store: &dyn Storage) -> StdResult<StandaloneState> {
        self.base_state.load(store)
    }
}

#[cfg(test)]
mod tests {
    use abstract_testing::prelude::*;

    use crate::mock::MockStandaloneContract;

    #[test]
    fn builder() {
        let app = MockStandaloneContract::new(TEST_MODULE_ID, TEST_VERSION, None);
        assert_eq!(app.module_id(), TEST_MODULE_ID);
        assert_eq!(app.version(), TEST_VERSION);
    }
}
