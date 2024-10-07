//! # Structs that implement a feature trait
//!
//! Feature objects are objects that store sufficient data to unlock some functionality.
//! These objects are mostly used internally to easy re-use application code without
//! requiring the usage of a base contract.

pub use abstract_std::objects::{ans_host::AnsHost, registry::RegistryContract};
use abstract_std::{registry::Account, REGISTRY};
use cosmwasm_std::{Deps, Env};

use crate::{
    features::{AccountIdentification, ModuleIdentification},
    std::ACCOUNT,
    AbstractSdkResult,
};

impl AccountIdentification for Account {
    fn account(&self, _deps: Deps) -> AbstractSdkResult<Account> {
        Ok(self.clone())
    }
}

impl ModuleIdentification for Account {
    /// Any actions executed by the core will be by the proxy address
    fn module_id(&self) -> &'static str {
        ACCOUNT
    }
}

impl crate::features::AbstractRegistryAccess for RegistryContract {
    fn abstract_registry(&self, _deps: Deps, _env: &Env) -> AbstractSdkResult<RegistryContract> {
        Ok(self.clone())
    }
}

impl ModuleIdentification for RegistryContract {
    fn module_id(&self) -> abstract_std::objects::module::ModuleId<'static> {
        REGISTRY
    }
}

impl crate::features::AbstractNameService for AnsHost {
    fn ans_host(&self, _deps: Deps, _env: &Env) -> AbstractSdkResult<AnsHost> {
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use abstract_testing::prelude::*;
    use speculoos::prelude::*;

    use super::*;

    mod registry {
        use cosmwasm_std::testing::mock_dependencies;

        use super::*;
        use crate::features::AbstractRegistryAccess;

        #[test]
        fn test_registry() {
            let deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let vc = RegistryContract::new(&deps.api, &env).unwrap();

            assert_that!(vc.abstract_registry(deps.as_ref(), &env))
                .is_ok()
                .is_equal_to(vc);
        }
    }

    mod account {
        use cosmwasm_std::{testing::mock_dependencies, Addr};

        use super::*;

        #[test]
        fn test_account_addr() {
            let deps = mock_dependencies();
            let account = test_account(deps.api);

            assert_that!(account.account(deps.as_ref()))
                .is_ok()
                .is_equal_to(account);
        }

        #[test]
        fn should_identify_self_as_account() {
            let account = Account::new(Addr::unchecked("test"));

            assert_that!(account.module_id()).is_equal_to(ACCOUNT);
        }
    }
}
