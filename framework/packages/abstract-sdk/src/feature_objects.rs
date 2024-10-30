//! # Structs that implement a feature trait
//!
//! Feature objects are objects that store sufficient data to unlock some functionality.
//! These objects are mostly used internally to easy re-use application code without
//! requiring the usage of a base contract.

pub use abstract_std::objects::{ans_host::AnsHost, registry::RegistryContract};
use abstract_std::{registry::Account, ANS_HOST, REGISTRY};
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

impl ModuleIdentification for AnsHost {
    fn module_id(&self) -> abstract_std::objects::module::ModuleId<'static> {
        ANS_HOST
    }
}

#[cfg(test)]
mod tests {
    use abstract_unit_test_utils::prelude::*;
    use cosmwasm_std::testing::mock_dependencies;

    use super::*;

    mod registry {

        use super::*;
        use crate::features::AbstractRegistryAccess;

        #[coverage_helper::test]
        fn test_registry() {
            let deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let registry = RegistryContract::new(&deps.api, &env).unwrap();

            assert_eq!(
                registry.abstract_registry(deps.as_ref(), &env).unwrap(),
                registry
            );
            assert_eq!(registry.module_id(), REGISTRY);
        }
    }

    mod ans {

        use abstract_std::ANS_HOST;

        use super::*;
        use crate::features::AbstractNameService;

        #[coverage_helper::test]
        fn test_ans() {
            let deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let ans = AnsHost::new(&deps.api, &env).unwrap();

            assert_eq!(ans.ans_host(deps.as_ref(), &env).unwrap(), ans);
            assert_eq!(ans.module_id(), ANS_HOST);
        }
    }

    mod account {
        use super::*;

        #[coverage_helper::test]
        fn test_account_object() {
            let deps = mock_dependencies();
            let account = test_account(deps.api);

            assert_eq!(account.account(deps.as_ref()).unwrap(), account);
            assert_eq!(account.module_id(), ACCOUNT);
        }
    }
}
