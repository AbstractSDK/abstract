use abstract_sdk::{
    feature_objects::{AnsHost, RegistryContract},
    features::{AbstractNameService, AbstractRegistryAccess, AccountIdentification, Dependencies},
    AbstractSdkResult,
};
use abstract_std::registry::Account;
use cosmwasm_std::{Deps, Env};

use crate::StandaloneContract;

// ANCHOR: ans
impl AbstractNameService for StandaloneContract {
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        let state = self.load_state(deps.storage)?;
        let contract_info = deps
            .querier
            .query_wasm_contract_info(state.account.into_addr())?;
        Ok(AnsHost::new(deps, contract_info.code_id)?)
    }
}
// ANCHOR_END: ans

impl AbstractRegistryAccess for StandaloneContract {
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<RegistryContract> {
        let state = self.load_state(deps.storage)?;
        let contract_info = deps
            .querier
            .query_wasm_contract_info(state.account.into_addr())?;
        Ok(RegistryContract::new(deps, contract_info.code_id)?)
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
    use abstract_std::objects::{
        namespace::{Namespace, ABSTRACT_NAMESPACE},
        ABSTRACT_ACCOUNT_ID,
    };
    use abstract_testing::prelude::*;

    use super::*;
    use crate::mock::*;

    #[coverage_helper::test]
    fn test_ans_host() -> StandaloneTestResult {
        let deps = mock_init(true);
        let env = mock_env_validated(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);

        let ans_host = BASIC_MOCK_STANDALONE.ans_host(deps.as_ref())?;

        assert_eq!(ans_host.address, abstr.ans_host);
        Ok(())
    }

    #[coverage_helper::test]
    fn test_abstract_registry() -> StandaloneTestResult {
        let deps = mock_init(true);
        let env = mock_env_validated(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);

        let abstract_registry = BASIC_MOCK_STANDALONE.abstract_registry(deps.as_ref())?;

        assert_eq!(abstract_registry.address, abstr.registry);
        Ok(())
    }

    #[coverage_helper::test]
    fn test_traits_generated() -> StandaloneTestResult {
        let mut deps = mock_init(true);
        let env = mock_env_validated(deps.api);
        let expected_account = test_account(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&expected_account, TEST_ACCOUNT_ID)
            .with_contract_map_entry(
                &abstr.registry,
                abstract_std::registry::state::NAMESPACES,
                (
                    &Namespace::unchecked(ABSTRACT_NAMESPACE),
                    ABSTRACT_ACCOUNT_ID,
                ),
            )
            .build();

        // AbstractNameService
        let host = BASIC_MOCK_STANDALONE
            .name_service(deps.as_ref())
            .host()
            .clone();
        assert_eq!(host, AnsHost::new(deps.as_ref(), 1)?);

        // AccountRegistry
        let binding = BASIC_MOCK_STANDALONE;
        let account_registry = binding.account_registry(deps.as_ref()).unwrap();
        let account = account_registry.account(&TEST_ACCOUNT_ID)?;
        assert_eq!(account, expected_account);

        let module_registry = binding.module_registry(deps.as_ref())?;
        let abstract_namespace =
            module_registry.query_namespace_raw(Namespace::unchecked(ABSTRACT_NAMESPACE))?;
        assert_eq!(abstract_namespace, Some(ABSTRACT_ACCOUNT_ID));

        Ok(())
    }

    #[coverage_helper::test]
    fn test_module_id() -> StandaloneTestResult {
        let module_id = BASIC_MOCK_STANDALONE.module_id();

        assert_eq!(module_id, TEST_MODULE_ID);

        Ok(())
    }
}
