use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    features::{AbstractNameService, AbstractRegistryAccess, AccountIdentification, Dependencies},
    AbstractSdkResult,
};
use abstract_std::version_control::Account;
use cosmwasm_std::Deps;

use crate::StandaloneContract;

// ANCHOR: ans
impl AbstractNameService for StandaloneContract {
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}
// ANCHOR_END: ans

impl AbstractRegistryAccess for StandaloneContract {
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<VersionControlContract> {
        Ok(self.base_state.load(deps.storage)?.version_control)
    }
}

impl AccountIdentification for StandaloneContract {
    fn account(&self, deps: Deps) -> AbstractSdkResult<Account> {
        Ok(self.base_state.load(deps.storage)?.account)
    }
}

impl Dependencies for StandaloneContract {
    fn dependencies(&self) -> &'static [abstract_std::objects::dependency::StaticDependency] {
        self.dependencies
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_sdk::{AccountVerification, ModuleRegistryInterface};
    use abstract_testing::{mock_querier, prelude::*};
    use speculoos::prelude::*;

    use super::*;
    use crate::mock::*;

    #[test]
    fn test_ans_host() -> StandaloneTestResult {
        let deps = mock_init();
        let abstr = AbstractMockAddrs::new(deps.api);

        let ans_host = BASIC_MOCK_STANDALONE.ans_host(deps.as_ref())?;

        assert_that!(ans_host.address).is_equal_to(abstr.ans_host);
        Ok(())
    }

    #[test]
    fn test_abstract_registry() -> StandaloneTestResult {
        let deps = mock_init();
        let abstr = AbstractMockAddrs::new(deps.api);

        let abstract_registry = BASIC_MOCK_STANDALONE.abstract_registry(deps.as_ref())?;

        assert_that!(abstract_registry.address).is_equal_to(abstr.version_control);
        Ok(())
    }

    #[test]
    fn test_traits_generated() -> StandaloneTestResult {
        let mut deps = mock_init();
        deps.querier = mock_querier(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);

        // AbstractNameService
        let host = BASIC_MOCK_STANDALONE
            .name_service(deps.as_ref())
            .host()
            .clone();
        assert_eq!(host, AnsHost::new(abstr.ans_host));

        // AccountRegistry
        // TODO: Why rust forces binding on static object what
        let binding = BASIC_MOCK_STANDALONE;
        let account_registry = binding.account_registry(deps.as_ref()).unwrap();
        let base = account_registry.account_base(&TEST_ACCOUNT_ID)?;
        assert_eq!(base, abstr.account);

        // TODO: Make some of the module_registry queries raw as well?
        let _module_registry = BASIC_MOCK_STANDALONE.module_registry(deps.as_ref());
        // _module_registry.query_namespace(Namespace::new(TEST_NAMESPACE)?)?;

        Ok(())
    }

    #[test]
    fn test_module_id() -> StandaloneTestResult {
        let module_id = BASIC_MOCK_STANDALONE.module_id();

        assert_that!(module_id).is_equal_to(TEST_MODULE_ID);

        Ok(())
    }
}
