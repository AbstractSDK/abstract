use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    features::{
        AbstractNameService, AbstractRegistryAccess, AccountExecutor, AccountIdentification,
    },
    AbstractSdkResult,
};
use cosmwasm_std::{Addr, Deps};

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
    fn proxy_address(&self, deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.base_state.load(deps.storage)?.proxy_address)
    }
}

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > AccountExecutor
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
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
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<VersionControlContract> {
        Ok(self.base_state.load(deps.storage)?.version_control)
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
    fn test_ans_host() -> AppTestResult {
        let deps = mock_init();
        let abstr = AbstractMockAddrs::new(deps.api);

        let ans_host = MOCK_APP_WITH_DEP.ans_host(deps.as_ref())?;

        assert_that!(ans_host.address).is_equal_to(abstr.ans_host);
        Ok(())
    }

    #[test]
    fn test_abstract_registry() -> AppTestResult {
        let deps = mock_init();
        let abstr = AbstractMockAddrs::new(deps.api);

        let abstract_registry = MOCK_APP_WITH_DEP.abstract_registry(deps.as_ref())?;

        assert_that!(abstract_registry.address).is_equal_to(abstr.version_control);
        Ok(())
    }

    #[test]
    fn test_traits_generated() -> AppTestResult {
        let mut deps = mock_init();
        deps.querier = mock_querier(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);
        let test_account_base = abstr.account;
        // Account identification
        let base = MOCK_APP_WITH_DEP.account_base(deps.as_ref())?;
        assert_eq!(base, test_account_base.clone());

        // AbstractNameService
        let host = MOCK_APP_WITH_DEP.name_service(deps.as_ref()).host().clone();
        assert_eq!(host, AnsHost::new(abstr.ans_host));

        // AccountRegistry
        // TODO: really rust forces binding CONST variable here?
        // It's because of returning Result, most likely polonius bug
        let binding = MOCK_APP_WITH_DEP;
        let account_registry = binding.account_registry(deps.as_ref()).unwrap();
        let base = account_registry.account_base(&TEST_ACCOUNT_ID)?;
        assert_eq!(base, test_account_base);

        // TODO: Make some of the module_registry queries raw as well?
        let _module_registry = MOCK_APP_WITH_DEP.module_registry(deps.as_ref());
        // _module_registry.query_namespace(Namespace::new(TEST_NAMESPACE)?)?;

        Ok(())
    }

    #[test]
    fn test_proxy_address() -> AppTestResult {
        let deps = mock_init();
        let base = test_account_base(deps.api);

        let proxy_address = MOCK_APP_WITH_DEP.proxy_address(deps.as_ref())?;

        assert_that!(proxy_address).is_equal_to(base.proxy);

        Ok(())
    }

    #[test]
    fn test_module_id() -> AppTestResult {
        let module_id = MOCK_APP_WITH_DEP.module_id();

        assert_that!(module_id).is_equal_to(TEST_WITH_DEP_MODULE_ID);

        Ok(())
    }
}
