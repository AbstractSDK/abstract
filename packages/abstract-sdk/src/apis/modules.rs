//! # Module
//! The Module interface provides helper functions to execute functions on other modules installed on the OS.

use crate::{
    features::{Dependencies, Identification},
    AbstractSdkResult,
};
use abstract_os::manager::state::OS_MODULES;
use cosmwasm_std::{Addr, Deps, QueryRequest, WasmQuery};
use cw2::{ContractVersion, CONTRACT};
use os::objects::module::ModuleId;

/// Interact with other modules on the OS.
pub trait ModuleInterface: Identification + Dependencies {
    fn modules<'a>(&'a self, deps: Deps<'a>) -> Modules<Self> {
        Modules { base: self, deps }
    }
}

impl<T> ModuleInterface for T where T: Identification + Dependencies {}

#[derive(Clone)]
pub struct Modules<'a, T: ModuleInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ModuleInterface> Modules<'a, T> {
    /// Retrieve the address of an application in this OS.
    /// This should **not** be used to execute messages on an `Api`.
    /// Use `Modules::api_request(..)` instead.
    pub fn module_address(&self, module_id: ModuleId) -> AbstractSdkResult<Addr> {
        let manager_addr = self.base.manager_address(self.deps)?;
        let maybe_module_addr = OS_MODULES.query(&self.deps.querier, manager_addr, module_id)?;
        let Some(module_addr) = maybe_module_addr else {
            return Err(crate::AbstractSdkError::MissingModule { module: module_id.to_string() });
        };
        Ok(module_addr)
    }

    /// Retrieve the version of an application in this OS.
    /// Note: this method makes use of the Cw2 query and may not coincide with the version of the
    /// module listed in VersionControl.
    pub fn module_version(&self, module_id: ModuleId) -> AbstractSdkResult<ContractVersion> {
        let module_address = self.module_address(module_id)?;
        let req = QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: module_address.into(),
            key: CONTRACT.as_slice().into(),
        });
        self.deps
            .querier
            .query::<ContractVersion>(&req)
            .map_err(Into::into)
    }

    pub fn assert_module_dependency(&self, module_id: ModuleId) -> AbstractSdkResult<()> {
        let is_dependency = Dependencies::dependencies(self.base)
            .iter()
            .map(|d| d.id)
            .any(|x| x == module_id);

        match is_dependency {
            true => Ok(()),
            false => Err(crate::AbstractSdkError::MissingDependency {
                module: module_id.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;

    use abstract_testing::prelude::TEST_MODULE_ID;
    use speculoos::prelude::*;

    mod assert_module_dependency {
        use super::*;
        use cosmwasm_std::testing::*;

        #[test]
        fn should_return_ok_if_dependency() {
            let deps = mock_dependencies();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let res = mods.assert_module_dependency(TEST_MODULE_ID);
            assert_that!(res).is_ok();
        }

        #[test]
        fn should_return_err_if_not_dependency() {
            let deps = mock_dependencies();
            let app = MockModule::new();

            let mods = app.modules(deps.as_ref());

            let fake_module = "lol_no_chance";
            let res = mods.assert_module_dependency(fake_module);

            assert_that!(res).is_err().matches(|e| {
                e.to_string()
                    .contains(&format!("{fake_module} is not a dependency"))
            });
        }
    }
}
