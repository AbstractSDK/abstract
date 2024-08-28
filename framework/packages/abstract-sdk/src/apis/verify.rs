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
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::AbstractSdkError;
    use abstract_std::{
        objects::{account::AccountTrace, module::ModuleId, version_control::VersionControlError},
        proxy::state::ACCOUNT_ID,
        version_control::state::ACCOUNT_ADDRESSES,
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    struct MockBinding {
        mock_api: MockApi,
    }

    impl AbstractRegistryAccess for MockBinding {
        fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<VersionControlContract> {
            let abstr = AbstractMockAddrs::new(self.mock_api);
            Ok(VersionControlContract::new(abstr.version_control))
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
            let not_base = AccountBase {
                manager: deps.api.addr_make("not_manager"),
                proxy: deps.api.addr_make("not_proxy"),
            };
            let base = test_account_base(deps.api);

            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                // Setup the addresses as if the Account was registered
                .account(&not_base, TEST_ACCOUNT_ID)
                // update the proxy to be proxy of a different Account
                .account(&base, SECOND_TEST_ACCOUNT_ID)
                .builder()
                .with_contract_item(&not_base.proxy, ACCOUNT_ID, &SECOND_TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding { mock_api: deps.api };

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_proxy(&not_base.proxy);

            let expected_err = AbstractSdkError::ApiQuery {
                api: AccountRegistry::<MockBinding>::api_id(),
                module_id: binding.module_id().to_owned(),
                error: Box::new(
                    VersionControlError::NotProxy(not_base.proxy, SECOND_TEST_ACCOUNT_ID).into(),
                ),
            };
            assert_eq!(res.unwrap_err(), expected_err);
        }

        #[test]
        fn inactive_account_fails() {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(&abstr.account.proxy, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_key(&abstr.version_control, ACCOUNT_ADDRESSES, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding { mock_api: deps.api };

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_proxy(&abstr.account.proxy);

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| {
                    e.to_string().contains(
                        &VersionControlError::UnknownAccountId {
                            account_id: TEST_ACCOUNT_ID,
                            registry_addr: abstr.version_control.clone(),
                        }
                        .to_string(),
                    )
                });
        }

        #[test]
        fn returns_core() {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(&base.proxy, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    &abstr.version_control,
                    ACCOUNT_ADDRESSES,
                    (&TEST_ACCOUNT_ID, base.clone()),
                )
                .build();

            let binding = MockBinding { mock_api: deps.api };

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_proxy(&base.proxy);

            assert_that!(res).is_ok().is_equal_to(base);
        }

        #[test]
        fn errors_when_not_manager_of_returned_os() {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(&base.proxy, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    &abstr.version_control,
                    ACCOUNT_ADDRESSES,
                    (
                        &TEST_ACCOUNT_ID,
                        AccountBase {
                            manager: base.manager,
                            proxy: deps.api.addr_make("not_proxy"),
                        },
                    ),
                )
                .build();

            let binding = MockBinding { mock_api: deps.api };

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_proxy(&base.proxy);

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
            let not_base = AccountBase {
                manager: deps.api.addr_make("not_manager"),
                proxy: deps.api.addr_make("not_proxy"),
            };
            let base = test_account_base(deps.api);

            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                // Setup the addresses as if the Account was registered
                .account(&not_base, TEST_ACCOUNT_ID)
                // update the proxy to be proxy of a different Account
                .account(&base, SECOND_TEST_ACCOUNT_ID)
                .builder()
                .with_contract_item(&not_base.manager, ACCOUNT_ID, &SECOND_TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding { mock_api: deps.api };

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_manager(&not_base.manager);

            let expected_err = AbstractSdkError::ApiQuery {
                api: AccountRegistry::<MockBinding>::api_id(),
                module_id: binding.module_id().to_owned(),
                error: Box::new(
                    VersionControlError::NotManager(not_base.manager, SECOND_TEST_ACCOUNT_ID)
                        .into(),
                ),
            };
            assert_eq!(res.unwrap_err(), expected_err);
        }

        #[test]
        fn inactive_account_fails() {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(&base.manager, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_key(&abstr.version_control, ACCOUNT_ADDRESSES, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding { mock_api: deps.api };

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_manager(&base.manager);

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| {
                    e.to_string().contains(
                        &VersionControlError::UnknownAccountId {
                            account_id: TEST_ACCOUNT_ID,
                            registry_addr: abstr.version_control.clone(),
                        }
                        .to_string(),
                    )
                });
        }

        #[test]
        fn returns_core() {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(&abstr.account.manager, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    &abstr.version_control,
                    ACCOUNT_ADDRESSES,
                    (&TEST_ACCOUNT_ID, abstr.account.clone()),
                )
                .build();

            let binding = MockBinding { mock_api: deps.api };

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_manager(&abstr.account.manager);

            assert_that!(res).is_ok().is_equal_to(abstr.account);
        }

        #[test]
        fn errors_when_not_manager_of_returned_os() {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(&abstr.account.manager, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    &abstr.version_control,
                    ACCOUNT_ADDRESSES,
                    (
                        &TEST_ACCOUNT_ID,
                        AccountBase {
                            manager: deps.api.addr_make("not_manager"),
                            proxy: abstr.account.proxy,
                        },
                    ),
                )
                .build();

            let binding = MockBinding { mock_api: deps.api };

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_manager(&abstr.account.manager);

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| {
                    e.to_string().contains(
                        &VersionControlError::NotManager(
                            abstr.account.manager.clone(),
                            TEST_ACCOUNT_ID,
                        )
                        .to_string(),
                    )
                });
        }
    }
}
