use abstract_sdk::{
    base::{ModuleId, ModuleMetadata, VersionString},
    features::ModuleIdentification,
    namespaces::{ADMIN_NAMESPACE, BASE_STATE},
};
use abstract_std::{
    objects::{module::ModuleInfo, nested_admin::NestedAdmin},
    standalone::StandaloneState,
    AbstractResult,
};
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;

/// The state variables for our AppContract.
pub struct StandaloneContract {
    pub admin: NestedAdmin<'static>,
    pub(crate) base_state: Item<'static, StandaloneState>,
    /// Static info about the contract, used for migration
    pub(crate) info: (ModuleId, VersionString, ModuleMetadata),
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
        }
    }

    pub fn module_id(&self) -> &str {
        self.info.0
    }

    pub fn version(&self) -> &str {
        self.info.1
    }

    pub fn module_info(&self) -> AbstractResult<ModuleInfo> {
        ModuleInfo::from_id(self.module_id(), self.version().into())
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
