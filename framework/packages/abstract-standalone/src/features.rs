use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    features::{AbstractNameService, AbstractRegistryAccess, AccountIdentification, Dependencies},
    AbstractSdkResult,
};
use cosmwasm_std::{Addr, Deps};

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
    fn proxy_address(&self, deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.base_state.load(deps.storage)?.proxy_address)
    }
}

impl Dependencies for StandaloneContract {
    fn dependencies(&self) -> &[abstract_std::objects::dependency::StaticDependency] {
        &self.dependencies
    }
}

#[cfg(test)]
mod test {
    use abstract_sdk::{AccountVerification, ModuleRegistryInterface};
    use abstract_std::version_control::AccountBase;
    use abstract_testing::{
        addresses::TEST_MODULE_ID,
        mock_querier,
        prelude::{TEST_ACCOUNT_ID, TEST_ANS_HOST, TEST_MANAGER, TEST_PROXY, TEST_VERSION_CONTROL},
    };
    use cosmwasm_std::Addr;
    use speculoos::prelude::*;

    use super::*;
    use crate::mock::*;

    #[test]
    fn test_ans_host() -> StandaloneTestResult {
        let deps = mock_init();

        let ans_host = BASIC_MOCK_STANDALONE.ans_host(deps.as_ref())?;

        assert_that!(ans_host.address).is_equal_to(Addr::unchecked(TEST_ANS_HOST));
        Ok(())
    }

    #[test]
    fn test_abstract_registry() -> StandaloneTestResult {
        let deps = mock_init();

        let abstract_registry = BASIC_MOCK_STANDALONE.abstract_registry(deps.as_ref())?;

        assert_that!(abstract_registry.address).is_equal_to(Addr::unchecked(TEST_VERSION_CONTROL));
        Ok(())
    }

    #[test]
    fn test_traits_generated() -> StandaloneTestResult {
        let mut deps = mock_init();
        deps.querier = mock_querier();
        let test_account_base = AccountBase {
            manager: Addr::unchecked(TEST_MANAGER),
            proxy: Addr::unchecked(TEST_PROXY),
        };

        // AbstractNameService
        let host = BASIC_MOCK_STANDALONE
            .name_service(deps.as_ref())
            .host()
            .clone();
        assert_eq!(host, AnsHost::new(Addr::unchecked(TEST_ANS_HOST)));

        // AccountRegistry
        let account_registry = BASIC_MOCK_STANDALONE
            .account_registry(deps.as_ref())
            .unwrap();
        let base = account_registry.account_base(&TEST_ACCOUNT_ID)?;
        assert_eq!(base, test_account_base);

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
