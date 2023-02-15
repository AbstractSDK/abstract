//! # Verification
//! The `Verify` struct provides helper functions that enable the contract to verify if the sender is an OS, OS admin, etc.
use crate::{features::AbstractRegistryAccess, AbstractSdkError, AbstractSdkResult};
use abstract_os::{
    manager::state::OS_ID,
    version_control::{state::OS_ADDRESSES, Core},
};
use cosmwasm_std::{Addr, Deps};

/// Verify if an addresses is associated with an OS.
pub trait OsVerification: AbstractRegistryAccess {
    fn os_registry<'a>(&'a self, deps: Deps<'a>) -> OsRegistry<Self> {
        OsRegistry { base: self, deps }
    }
}

impl<T> OsVerification for T where T: AbstractRegistryAccess {}

/// Endpoint for OS address verification
#[derive(Clone)]
pub struct OsRegistry<'a, T: OsVerification> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: OsVerification> OsRegistry<'a, T> {
    /// Verify if the provided manager address is indeed a user.
    pub fn assert_manager(&self, maybe_manager: &Addr) -> AbstractSdkResult<Core> {
        let os_id = OS_ID
            .query(&self.deps.querier, maybe_manager.clone())
            .map_err(|_| AbstractSdkError::FailedToQueryOsId {
                contract_addr: maybe_manager.clone(),
            })?;
        let vc_address = self.base.abstract_registry(self.deps)?;
        let maybe_os = OS_ADDRESSES.query(&self.deps.querier, vc_address.clone(), os_id)?;
        match maybe_os {
            None => Err(AbstractSdkError::UnknownOsId {
                os_id,
                version_control_addr: vc_address,
            }),
            Some(core) => {
                if &core.manager != maybe_manager {
                    Err(AbstractSdkError::NotManager(maybe_manager.clone(), os_id))
                } else {
                    Ok(core)
                }
            }
        }
    }

    /// Verify if the provided proxy address is indeed a user.
    pub fn assert_proxy(&self, maybe_proxy: &Addr) -> AbstractSdkResult<Core> {
        let os_id = OS_ID
            .query(&self.deps.querier, maybe_proxy.clone())
            .map_err(|_| AbstractSdkError::FailedToQueryOsId {
                contract_addr: maybe_proxy.clone(),
            })?;

        let vc_address = self.base.abstract_registry(self.deps)?;
        let maybe_os = OS_ADDRESSES.query(&self.deps.querier, vc_address.clone(), os_id)?;
        match maybe_os {
            None => Err(AbstractSdkError::UnknownOsId {
                os_id,
                version_control_addr: vc_address,
            }),
            Some(core) => {
                if &core.proxy != maybe_proxy {
                    Err(AbstractSdkError::NotProxy(maybe_proxy.clone(), os_id))
                } else {
                    Ok(core)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_testing::*;
    use cosmwasm_std::testing::*;

    use abstract_testing::{
        prelude::*, MockQuerierBuilder, TEST_OS_ID, TEST_PROXY, TEST_VERSION_CONTROL,
    };

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
            deps.querier = mocked_os_querier_builder()
                // Setup the addresses as if the OS was registered
                .os("not_manager", "not_proxy", TEST_OS_ID)
                // update the proxy to be proxy of a different OS
                .os(TEST_MANAGER, TEST_PROXY, 1)
                .builder()
                .with_contract_item("not_proxy", OS_ID, &1)
                .build();

            let binding = MockBinding;

            let res = binding
                .os_registry(deps.as_ref())
                .assert_proxy(&Addr::unchecked("not_proxy"));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::NotProxy(..)))
                .matches(|e| e.to_string().contains("not_proxy"));
        }

        #[test]
        fn inactive_os_fails() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, OS_ID, &TEST_OS_ID)
                .with_contract_map_key(TEST_VERSION_CONTROL, OS_ADDRESSES, TEST_OS_ID)
                .build();

            let binding = MockBinding;

            let res = binding
                .os_registry(deps.as_ref())
                .assert_proxy(&Addr::unchecked(TEST_PROXY));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::UnknownOsId { .. }))
                .matches(|e| e.to_string().contains("Unknown OS id 0"));
        }

        #[test]
        fn returns_core() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, OS_ID, &TEST_OS_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    OS_ADDRESSES,
                    (TEST_OS_ID, &test_core()),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .os_registry(deps.as_ref())
                .assert_proxy(&Addr::unchecked(TEST_PROXY));

            assert_that!(res).is_ok().is_equal_to(test_core());
        }

        #[test]
        fn errors_when_not_manager_of_returned_os() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, OS_ID, &TEST_OS_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    OS_ADDRESSES,
                    (
                        TEST_OS_ID,
                        &Core {
                            manager: Addr::unchecked(TEST_MANAGER),
                            proxy: Addr::unchecked("not_poxry"),
                        },
                    ),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .os_registry(deps.as_ref())
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
            deps.querier = mocked_os_querier_builder()
                // Setup the addresses as if the OS was registered
                .os("not_manager", "not_proxy", TEST_OS_ID)
                // update the proxy to be proxy of a different OS
                .os(TEST_MANAGER, TEST_PROXY, 1)
                .builder()
                .with_contract_item("not_manager", OS_ID, &1)
                .build();

            let binding = MockBinding;

            let res = binding
                .os_registry(deps.as_ref())
                .assert_manager(&Addr::unchecked("not_manager"));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::NotManager(..)))
                .matches(|e| e.to_string().contains("not_manager is not the Manager"));
        }

        #[test]
        fn inactive_os_fails() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_MANAGER, OS_ID, &TEST_OS_ID)
                .with_contract_map_key(TEST_VERSION_CONTROL, OS_ADDRESSES, TEST_OS_ID)
                .build();

            let binding = MockBinding;

            let res = binding
                .os_registry(deps.as_ref())
                .assert_manager(&Addr::unchecked(TEST_MANAGER));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::UnknownOsId { .. }))
                .matches(|e| {
                    e.to_string().contains(&format!(
                        "Unknown OS id {TEST_OS_ID} on version control {TEST_VERSION_CONTROL}"
                    ))
                });
        }

        #[test]
        fn returns_core() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_MANAGER, OS_ID, &TEST_OS_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    OS_ADDRESSES,
                    (TEST_OS_ID, &test_core()),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .os_registry(deps.as_ref())
                .assert_manager(&Addr::unchecked(TEST_MANAGER));

            assert_that!(res).is_ok().is_equal_to(test_core());
        }

        #[test]
        fn errors_when_not_manager_of_returned_os() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_MANAGER, OS_ID, &TEST_OS_ID)
                .with_contract_map_entry(
                    TEST_VERSION_CONTROL,
                    OS_ADDRESSES,
                    (
                        TEST_OS_ID,
                        &Core {
                            manager: Addr::unchecked("not_manager"),
                            proxy: Addr::unchecked(TEST_PROXY),
                        },
                    ),
                )
                .build();

            let binding = MockBinding;

            let res = binding
                .os_registry(deps.as_ref())
                .assert_manager(&Addr::unchecked(TEST_MANAGER));

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::NotManager(..)))
                .matches(|e| e.to_string().contains("not the Manager"))
                .matches(|e| e.to_string().contains(TEST_MANAGER));
        }
    }
}
