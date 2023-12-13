use crate::{state::ContractError, AppContract};
use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    features::{AbstractNameService, AbstractRegistryAccess, AccountIdentification, DepsAccess},
    AbstractSdkResult,
};
use cosmwasm_std::Addr;

// ANCHOR: ans
impl<
        'app,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > AbstractNameService
    for AppContract<
        'app,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(self.base_state.load(self.deps().storage)?.ans_host)
    }
}
// ANCHOR_END: ans

impl<
        'app,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > AccountIdentification
    for AppContract<
        'app,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(self.base_state.load(self.deps().storage)?.proxy_address)
    }
}

impl<
        'app,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > AbstractRegistryAccess
    for AppContract<
        'app,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn abstract_registry(&self) -> AbstractSdkResult<VersionControlContract> {
        Ok(self.base_state.load(self.deps().storage)?.version_control)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_core::version_control::AccountBase;
    use abstract_sdk::{AccountVerification, ModuleRegistryInterface};
    use abstract_testing::{
        mock_querier,
        prelude::{
            TEST_ACCOUNT_ID, TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_ID, TEST_PROXY,
            TEST_VERSION_CONTROL,
        },
    };
    use speculoos::prelude::*;

    use crate::mock::*;

    #[test]
    fn test_ans_host() -> AppTestResult {
        let deps = mock_init();

        let ans_host = mock_app((deps.as_ref(), mock_env()).into()).ans_host()?;

        assert_that!(ans_host.address).is_equal_to(Addr::unchecked(TEST_ANS_HOST));
        Ok(())
    }

    #[test]
    fn test_abstract_registry() -> AppTestResult {
        let deps = mock_init();

        let abstract_registry = mock_app((deps.as_ref(), mock_env()).into()).abstract_registry()?;

        assert_that!(abstract_registry.address).is_equal_to(Addr::unchecked(TEST_VERSION_CONTROL));
        Ok(())
    }

    #[test]
    fn test_traits_generated() -> AppTestResult {
        let mut deps = mock_init();
        deps.querier = mock_querier();
        let test_account_base = AccountBase {
            manager: Addr::unchecked(TEST_MANAGER),
            proxy: Addr::unchecked(TEST_PROXY),
        };
        // Account identification
        let base = mock_app((deps.as_ref(), mock_env()).into()).account_base()?;
        assert_eq!(base, test_account_base.clone());

        // AbstractNameService
        let host = mock_app((deps.as_ref(), mock_env()).into())
            .name_service()
            .host()
            .clone();
        assert_eq!(host, AnsHost::new(Addr::unchecked(TEST_ANS_HOST)));

        // AccountRegistry
        let base = mock_app((deps.as_ref(), mock_env()).into())
            .account_registry()
            .account_base(&TEST_ACCOUNT_ID)?;
        assert_eq!(base, test_account_base);

        // TODO: Make some of the module_registry queries raw as well?
        let _module_registry = mock_app((deps.as_ref(), mock_env()).into()).module_registry();
        // _module_registry.query_namespace(Namespace::new(TEST_NAMESPACE)?)?;

        Ok(())
    }

    #[test]
    fn test_proxy_address() -> AppTestResult {
        let deps = mock_init();

        let proxy_address = mock_app((deps.as_ref(), mock_env()).into()).proxy_address()?;

        assert_that!(proxy_address).is_equal_to(Addr::unchecked(TEST_PROXY));

        Ok(())
    }

    #[test]
    fn test_module_id() -> AppTestResult {
        let deps = mock_init();
        let module_id = mock_app((deps.as_ref(), mock_env()).into()).module_id();

        assert_that!(module_id).is_equal_to(TEST_MODULE_ID.to_string());

        Ok(())
    }
}
