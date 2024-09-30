use abstract_sdk::{
    feature_objects::{AnsHost, RegistryContract},
    features::{AbstractNameService, AbstractRegistryAccess, AccountIdentification},
    AbstractSdkResult,
};
use abstract_std::registry::Account;
use cosmwasm_std::Deps;

use crate::{state::ContractError, AppContract};

// ANCHOR: ans
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > AbstractNameService
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}
// ANCHOR_END: ans

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > AccountIdentification
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    fn account(&self, deps: Deps) -> AbstractSdkResult<Account> {
        Ok(self.base_state.load(deps.storage)?.account)
    }
}

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > AbstractRegistryAccess
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<RegistryContract> {
        Ok(self.base_state.load(deps.storage)?.registry)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_sdk::{AccountVerification, ModuleRegistryInterface};
    use abstract_testing::prelude::*;

    use super::*;
    use crate::mock::*;

    #[test]
    fn test_ans_host() -> AppTestResult {
        let deps = mock_init();
        let abstr = AbstractMockAddrs::new(deps.api);

        let ans_host = MOCK_APP_WITH_DEP.ans_host(deps.as_ref())?;

        assert_eq!(ans_host.address, abstr.ans_host);
        Ok(())
    }

    #[test]
    fn test_abstract_registry() -> AppTestResult {
        let deps = mock_init();
        let abstr = AbstractMockAddrs::new(deps.api);

        let abstract_registry = MOCK_APP_WITH_DEP.abstract_registry(deps.as_ref())?;

        assert_eq!(abstract_registry.address, abstr.registry);
        Ok(())
    }

    #[test]
    fn test_traits_generated() -> AppTestResult {
        let mut deps = mock_init();
        let test_account = test_account(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&test_account, TEST_ACCOUNT_ID)
            .build();
        // Account identification
        let base = MOCK_APP_WITH_DEP.account(deps.as_ref())?;
        assert_eq!(base, test_account.clone());

        // AbstractNameService
        let host = MOCK_APP_WITH_DEP.name_service(deps.as_ref()).host().clone();
        assert_eq!(host, AnsHost::new(&deps.api)?);

        // AccountRegistry
        // TODO: really rust forces binding CONST variable here?
        // It's because of returning Result, most likely polonius bug
        let binding = MOCK_APP_WITH_DEP;
        let account_registry = binding.account_registry(deps.as_ref())?;
        let base = account_registry.account(&TEST_ACCOUNT_ID)?;
        assert_eq!(base, test_account);

        // TODO: Make some of the module_registry queries raw as well?
        let _module_registry = MOCK_APP_WITH_DEP.module_registry(deps.as_ref());
        // _module_registry.query_namespace(Namespace::new(TEST_NAMESPACE)?)?;

        Ok(())
    }

    #[test]
    fn test_proxy_address() -> AppTestResult {
        let deps = mock_init();
        let expected_account = test_account(deps.api);

        let account = MOCK_APP_WITH_DEP.account(deps.as_ref())?;

        assert_eq!(account, expected_account);

        Ok(())
    }

    #[test]
    fn test_module_id() -> AppTestResult {
        let module_id = MOCK_APP_WITH_DEP.module_id();

        assert_eq!(module_id, TEST_WITH_DEP_MODULE_ID);
        Ok(())
    }
}
