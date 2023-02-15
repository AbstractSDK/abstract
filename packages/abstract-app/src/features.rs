use crate::{AppContract, AppError};
use abstract_sdk::{
    base::ContractName,
    feature_objects::AnsHost,
    features::{AbstractNameService, Identification, ModuleIdentification},
    AbstractSdkResult,
};
use cosmwasm_std::{Addr, Deps};

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > AbstractNameService
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Identification
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    fn proxy_address(&self, deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.base_state.load(deps.storage)?.proxy_address)
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > ModuleIdentification
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    fn module_id(&self) -> ContractName {
        self.contract.info().0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_testing::{TEST_ANS_HOST, TEST_MODULE_ID, TEST_PROXY};

    use crate::test_common::*;

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
