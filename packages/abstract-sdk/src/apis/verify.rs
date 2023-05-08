//! # Verification
//! The `Verify` struct provides helper functions that enable the contract to verify if the sender is an Abstract Account, Account admin, etc.
use crate::{features::AbstractRegistryAccess, AbstractSdkError, AbstractSdkResult};
use abstract_core::{
    manager::state::ACCOUNT_ID,
    version_control::{state::ACCOUNT_ADDRESSES, AccountBase},
};
use cosmwasm_std::{Addr, Deps};

/// Verify if an addresses is associated with an Abstract Account.
pub trait OsVerification: AbstractRegistryAccess {
    fn account_registry<'a>(&'a self, deps: Deps<'a>) -> OsRegistry<Self> {
        OsRegistry { base: self, deps }
    }
}

impl<T> OsVerification for T where T: AbstractRegistryAccess {}

/// Endpoint for Account address verification
#[derive(Clone)]
pub struct OsRegistry<'a, T: OsVerification> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: OsVerification> OsRegistry<'a, T> {
    /// Verify if the provided manager address is indeed a user.
    pub fn assert_manager(&self, maybe_manager: &Addr) -> AbstractSdkResult<AccountBase> {
        let account_id = self.account_id(maybe_manager)?;
        let account_base = self.account_base(account_id)?;
        if account_base.manager != maybe_manager {
            Err(AbstractSdkError::NotManager(
                maybe_manager.clone(),
                account_id,
            ))
        } else {
            Ok(account_base)
        }
    }

    /// Verify if the provided proxy address is indeed a user.
    pub fn assert_proxy(&self, maybe_proxy: &Addr) -> AbstractSdkResult<AccountBase> {
        let account_id = self.account_id(maybe_proxy)?;
        let account_base = self.account_base(account_id)?;
        if account_base.proxy != maybe_proxy {
            Err(AbstractSdkError::NotProxy(maybe_proxy.clone(), account_id))
        } else {
            Ok(account_base)
        }
    }

    pub fn proxy_address(&self, account_id: u32) -> AbstractSdkResult<Addr> {
        self.account_base(account_id)
            .map(|account_base| account_base.proxy)
    }

    pub fn manager_address(&self, account_id: u32) -> AbstractSdkResult<Addr> {
        self.account_base(account_id)
            .map(|account_base| account_base.manager)
    }

    pub fn account_base(&self, account_id: u32) -> AbstractSdkResult<AccountBase> {
        let maybe_account = ACCOUNT_ADDRESSES.query(
            &self.deps.querier,
            self.base.abstract_registry(self.deps)?,
            account_id,
        )?;
        match maybe_account {
            None => Err(AbstractSdkError::UnknownAccountId {
                account_id,
                version_control_addr: self.base.abstract_registry(self.deps)?,
            }),
            Some(account_base) => Ok(account_base),
        }
    }

    fn account_id(&self, maybe_core_contract_addr: &Addr) -> AbstractSdkResult<u32> {
        ACCOUNT_ID
            .query(&self.deps.querier, maybe_core_contract_addr.clone())
            .map_err(|_| AbstractSdkError::FailedToQueryAccountId {
                contract_addr: maybe_core_contract_addr.clone(),
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_testing::*;
    use cosmwasm_std::testing::*;

    use abstract_testing::prelude::*;
    use speculoos::prelude::*;

    struct MockBinding;

    impl AbstractRegistryAccess for MockBinding {
        fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
            Ok(Addr::unchecked(TEST_VERSION_CONTROL))
        }
    }

    mod assert_proxy {

        use super::*;

        #[test]
        fn not_proxy_fails() {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder()
                // Setup the addresses as if the Account was registered
                .account("not_manager", "not_proxy", TEST_ACCOUNT_ID)
                // update the proxy to be proxy of a different Account
                .account(TEST_MANAGER, TEST_PROXY, 1)
                .builder()
                .with_contract_item("not_proxy", ACCOUNT_ID, &1)
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .assert_proxy(&Addr::unchecked("not_proxy"));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::NotProxy(..)))
                .matches(|e| e.to_string().contains("not_proxy"));
        }

        #[test]
        fn inactive_account_fails() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_key(TEST_VERSION_CONTROL, ACCOUNT_ADDRESSES, TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .assert_proxy(&Addr::unchecked(TEST_PROXY));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::UnknownAccountId { .. }))
                .matches(|e| e.to_string().contains("Unknown Account id 0"));
        }

        #[test]
        fn returns_core() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    ACCOUNT_ADDRESSES,
                    (TEST_ACCOUNT_ID, test_account_base()),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
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
                        TEST_ACCOUNT_ID,
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
                .account(TEST_MANAGER, TEST_PROXY, 1)
                .builder()
                .with_contract_item("not_manager", ACCOUNT_ID, &1)
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .assert_manager(&Addr::unchecked("not_manager"));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::NotManager(..)))
                .matches(|e| e.to_string().contains("not_manager is not the Manager"));
        }

        #[test]
        fn inactive_account_fails() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_MANAGER, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_key(TEST_VERSION_CONTROL, ACCOUNT_ADDRESSES, TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
                .assert_manager(&Addr::unchecked(TEST_MANAGER));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::UnknownAccountId { .. }))
                .matches(|e| {
                    e.to_string().contains(&format!(
                        "Unknown Account id {TEST_ACCOUNT_ID} on version control {TEST_VERSION_CONTROL}"
                    ))
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
                    (TEST_ACCOUNT_ID, test_account_base()),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .account_registry(deps.as_ref())
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
                        TEST_ACCOUNT_ID,
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
                .assert_manager(&Addr::unchecked(TEST_MANAGER));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::NotManager(..)))
                .matches(|e| e.to_string().contains("not the Manager"))
                .matches(|e| e.to_string().contains(TEST_MANAGER));
        }
    }
}
