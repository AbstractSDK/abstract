use crate::{state::ContractError, AppContract};
use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    features::{AbstractNameService, AbstractRegistryAccess, AccountIdentification},
    AbstractSdkResult,
};
use cosmwasm_std::{Addr, Deps};

// ANCHOR: ans
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > AbstractNameService
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
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
        ReceiveMsg,
        SudoMsg,
    > AccountIdentification
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
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
        ReceiveMsg,
        SudoMsg,
    > AbstractRegistryAccess
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<VersionControlContract> {
        Ok(self.base_state.load(deps.storage)?.version_control)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_sdk::features::ModuleIdentification;
    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_MODULE_ID, TEST_PROXY};
    use speculoos::prelude::*;

    use crate::mock::*;

    #[test]
    fn test_ans_host() -> AppTestResult {
        let deps = mock_init();

        let ans_host = MOCK_APP.ans_host(deps.as_ref())?;

        assert_that!(ans_host.address).is_equal_to(Addr::unchecked(TEST_ANS_HOST));

        Ok(())
    }

    #[test]
    fn test_proxy_address() -> AppTestResult {
        let deps = mock_init();

        let proxy_address = MOCK_APP.proxy_address(deps.as_ref())?;

        assert_that!(proxy_address).is_equal_to(Addr::unchecked(TEST_PROXY));

        Ok(())
    }

    #[test]
    fn test_module_id() -> AppTestResult {
        let module_id = MOCK_APP.module_id();

        assert_that!(module_id).is_equal_to(TEST_MODULE_ID);

        Ok(())
    }
}
