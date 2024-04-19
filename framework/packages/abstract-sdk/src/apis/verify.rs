//! # Verification
//! The `Verify` struct provides helper functions that enable the contract to verify if the sender is an Abstract Account, Account admin, etc.
use abstract_std::{
    objects::{version_control::VersionControlContract, AccountId},
    version_control::AccountBase,
};
use cosmwasm_std::{Addr, Deps};

use super::{AbstractApi, ApiIdentification};
use crate::{
    cw_helpers::ApiQuery,
    features::{AbstractRegistryAccess, ModuleIdentification},
    AbstractSdkResult,
};

/// Verify if an addresses is associated with an Abstract Account.
pub trait AccountVerification: AbstractRegistryAccess + ModuleIdentification {
    /**
        API for querying and verifying a sender's identity in the context of Abstract Accounts.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let acc_registry: AccountRegistry<MockModule>  = module.account_registry(deps.as_ref()).unwrap();
        ```
    */
    fn account_registry<'a>(&'a self, deps: Deps<'a>) -> AbstractSdkResult<AccountRegistry<Self>> {
        let vc = self.abstract_registry(deps)?;
        Ok(AccountRegistry {
            base: self,
            deps,
            vc,
        })
    }
}

impl<T> AccountVerification for T where T: AbstractRegistryAccess + ModuleIdentification {}

impl<'a, T: AccountVerification> AbstractApi<T> for AccountRegistry<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

impl<'a, T: AccountVerification> ApiIdentification for AccountRegistry<'a, T> {
    fn api_id() -> String {
        "AccountRegistry".to_owned()
    }
}

/**
    API for querying and verifying a sender's identity in the context of Abstract Accounts.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let acc_registry: AccountRegistry<MockModule>  = module.account_registry(deps.as_ref()).unwrap();
    ```
*/
#[derive(Clone)]
pub struct AccountRegistry<'a, T: AccountVerification> {
    base: &'a T,
    deps: Deps<'a>,
    vc: VersionControlContract,
}

impl<'a, T: AccountVerification> AccountRegistry<'a, T> {
    /// Verify if the provided manager address is indeed a user.
    pub fn assert_manager(&self, maybe_manager: &Addr) -> AbstractSdkResult<AccountBase> {
        self.vc
            .assert_manager(maybe_manager, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Verify if the provided proxy address is indeed a user.
    pub fn assert_proxy(&self, maybe_proxy: &Addr) -> AbstractSdkResult<AccountBase> {
        self.vc
            .assert_proxy(maybe_proxy, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Get the proxy address for a given account id.
    pub fn proxy_address(&self, account_id: &AccountId) -> AbstractSdkResult<Addr> {
        self.account_base(account_id)
            .map(|account_base| account_base.proxy)
    }

    /// Get the manager address for a given account id.
    pub fn manager_address(&self, account_id: &AccountId) -> AbstractSdkResult<Addr> {
        self.account_base(account_id)
            .map(|account_base| account_base.manager)
    }

    /// Get the account base for a given account id.
    pub fn account_base(&self, account_id: &AccountId) -> AbstractSdkResult<AccountBase> {
        self.vc
            .account_base(account_id, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Get AccountId for given manager or proxy address.
    pub fn account_id(&self, maybe_core_contract_addr: &Addr) -> AbstractSdkResult<AccountId> {
        self.vc
            .account_id(maybe_core_contract_addr, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Get namespace registration fee
    pub fn namespace_registration_fee(&self) -> AbstractSdkResult<Option<cosmwasm_std::Coin>> {
        self.vc
            .namespace_registration_fee(&self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }
}

#[cfg(test)]
mod test {

    use abstract_std::{
        objects::{
            account::AccountTrace,
            module::ModuleId,
            version_control::{VersionControlContract, VersionControlError},
        },
        proxy::state::ACCOUNT_ID,
        version_control::state::ACCOUNT_ADDRESSES,
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    use super::*;
    use crate::AbstractSdkError;

    struct MockBinding;

    impl AbstractRegistryAccess for MockBinding {
        fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<VersionControlContract> {
            Ok(VersionControlContract::new(Addr::unchecked(
                TEST_VERSION_CONTROL,
            )))
        }
    }

    impl ModuleIdentification for MockBinding {
        fn module_id(&self) -> ModuleId<'static> {
            ModuleId::from("module")
        }
    }

    pub const SECOND_TEST_ACCOUNT_ID: AccountId = AccountId::const_new(2, AccountTrace::Local);

    mod assert_proxy {

        use super::*;

        #[test]
        fn not_proxy_fails() {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder()
                // Setup the addresses as if the Account was registered
                .account("not_manager", "not_proxy", TEST_ACCOUNT_ID)
                // update the proxy to be proxy of a different Account
                .account(TEST_MANAGER, TEST_PROXY, SECOND_TEST_ACCOUNT_ID)
                .builder()
                .with_contract_item("not_proxy", ACCOUNT_ID, &SECOND_TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_proxy(&Addr::unchecked("not_proxy"));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| e.to_string().contains("not_proxy"));
        }

        #[test]
        fn inactive_account_fails() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_key(TEST_VERSION_CONTROL, ACCOUNT_ADDRESSES, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_proxy(&Addr::unchecked(TEST_PROXY));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| {
                    e.to_string().contains(
                        &VersionControlError::UnknownAccountId {
                            account_id: TEST_ACCOUNT_ID,
                            registry_addr: Addr::unchecked(TEST_VERSION_CONTROL),
                        }
                        .to_string(),
                    )
                });
        }

        #[test]
        fn returns_core() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    ACCOUNT_ADDRESSES,
                    (&TEST_ACCOUNT_ID, test_account_base()),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_proxy(&Addr::unchecked(TEST_PROXY));

            assert_that!(res).is_ok().is_equal_to(test_account_base());
        }

        #[test]
        fn errors_when_not_manager_of_returned_os() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    ACCOUNT_ADDRESSES,
                    (
                        &TEST_ACCOUNT_ID,
                        AccountBase {
                            manager: Addr::unchecked(TEST_MANAGER),
                            proxy: Addr::unchecked("not_poxry"),
                        },
                    ),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_proxy(&Addr::unchecked(TEST_PROXY));

            assert_that!(res)
                .is_err()
                // .matches(|e| matches!(e, AbstractSdkError::Std(StdError::GenericErr { .. })))
                .matches(|e| e.to_string().contains("not the Proxy"));
        }
    }

    mod assert_manager {
        use super::*;

        #[test]
        fn not_manager_fails() {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder()
                // Setup the addresses as if the Account was registered
                .account("not_manager", "not_proxy", TEST_ACCOUNT_ID)
                // update the proxy to be proxy of a different Account
                .account(TEST_MANAGER, TEST_PROXY, SECOND_TEST_ACCOUNT_ID)
                .builder()
                .with_contract_item("not_manager", ACCOUNT_ID, &SECOND_TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_manager(&Addr::unchecked("not_manager"));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| e.to_string().contains("not_manager is not the Manager"));
        }

        #[test]
        fn inactive_account_fails() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_MANAGER, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_key(TEST_VERSION_CONTROL, ACCOUNT_ADDRESSES, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_manager(&Addr::unchecked(TEST_MANAGER));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| {
                    e.to_string().contains(
                        &VersionControlError::UnknownAccountId {
                            account_id: TEST_ACCOUNT_ID,
                            registry_addr: Addr::unchecked(TEST_VERSION_CONTROL),
                        }
                        .to_string(),
                    )
                });
        }

        #[test]
        fn returns_core() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_MANAGER, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    ACCOUNT_ADDRESSES,
                    (&TEST_ACCOUNT_ID, test_account_base()),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_manager(&Addr::unchecked(TEST_MANAGER));

            assert_that!(res).is_ok().is_equal_to(test_account_base());
        }

        #[test]
        fn errors_when_not_manager_of_returned_os() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_MANAGER, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    ACCOUNT_ADDRESSES,
                    (
                        &TEST_ACCOUNT_ID,
                        AccountBase {
                            manager: Addr::unchecked("not_manager"),
                            proxy: Addr::unchecked(TEST_PROXY),
                        },
                    ),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_manager(&Addr::unchecked(TEST_MANAGER));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| {
                    e.to_string().contains(
                        &VersionControlError::NotManager(
                            Addr::unchecked(TEST_MANAGER),
                            TEST_ACCOUNT_ID,
                        )
                        .to_string(),
                    )
                });
        }
    }
}
