//! # Module
//! The Module interface provides helper functions to execute functions on other modules installed on the Account.

use abstract_std::{account::state::ACCOUNT_MODULES, objects::module::ModuleId};
use cosmwasm_std::{Addr, Deps, QueryRequest, WasmQuery};
use cw2::{ContractVersion, CONTRACT};

use super::{AbstractApi, ApiIdentification};
use crate::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult,
};

/// Interact with other modules on the Account.
pub trait ModuleInterface: AccountIdentification + Dependencies + ModuleIdentification {
    /**
        API for retrieving information about installed modules.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # use abstract_testing::prelude::*;
        # let deps = mock_dependencies();
        # let account = admin_account(deps.api);
        # let module = MockModule::new(deps.api, account);

        let modules: Modules<MockModule>  = module.modules(deps.as_ref());
        ```
    */
    fn modules<'a>(&'a self, deps: Deps<'a>) -> Modules<Self> {
        Modules { base: self, deps }
    }
}

impl<T> ModuleInterface for T where T: AccountIdentification + Dependencies + ModuleIdentification {}

impl<'a, T: ModuleInterface> AbstractApi<T> for Modules<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

impl<'a, T: ModuleInterface> ApiIdentification for Modules<'a, T> {
    fn api_id() -> String {
        "Modules".to_owned()
    }
}

/**
    API for retrieving information about installed modules.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # use abstract_testing::prelude::*;
    # let deps = mock_dependencies();
    # let account = admin_account(deps.api);
    # let module = MockModule::new(deps.api, account);

    let modules: Modules<MockModule>  = module.modules(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Modules<'a, T: ModuleInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ModuleInterface> Modules<'a, T> {
    /// Retrieve the address of an application in this Account.
    /// This should **not** be used to execute messages on an `Api`.
    /// Use `Modules::api_request(..)` instead.
    pub fn module_address(&self, module_id: ModuleId) -> AbstractSdkResult<Addr> {
        let account_addr = self.base.account(self.deps)?;
        let maybe_module_addr =
            ACCOUNT_MODULES.query(&self.deps.querier, account_addr.into_addr(), module_id)?;
        let Some(module_addr) = maybe_module_addr else {
            return Err(crate::AbstractSdkError::MissingModule {
                module: module_id.to_string(),
            });
        };
        Ok(module_addr)
    }

    /// Retrieve the version of an application in this Account.
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

    /// Assert that a module is a dependency of this module.
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
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_testing::abstract_mock_querier_builder;
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    use super::*;
    use crate::mock_module::*;

    pub fn setup() -> (MockDeps, MockModule) {
        let mut deps = mock_dependencies();
        let account = test_account_base(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let app = MockModule::new(deps.api, account.clone());

        (deps, app)
    }

    mod assert_module_dependency {
        use super::*;

        #[test]
        fn should_return_ok_if_dependency() {
            let (deps, app) = setup();

            let mods = app.modules(deps.as_ref());

            let res = mods.assert_module_dependency(TEST_MODULE_ID);
            assert_that!(res).is_ok();
        }

        #[test]
        fn should_return_err_if_not_dependency() {
            let (deps, app) = setup();

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
