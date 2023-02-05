use abstract_os::{
    objects::common_namespace::ADMIN_NAMESPACE, proxy::state::OS_ID, version_control::Core,
};
use cosmwasm_std::{Addr, Deps, StdError, StdResult};
use cw_storage_plus::Item;
use os::objects::OsId;

const MANAGER: Item<'_, Option<Addr>> = Item::new(ADMIN_NAMESPACE);

/// A trait that enables the identification of an OS.
/// This includes the manager, porxy, core (manager/proxy) and osId.
/// TODO: rename OsIdentification
pub trait Identification: Sized {
    fn proxy_address(&self, deps: Deps) -> StdResult<Addr>;
    fn manager_address(&self, deps: Deps) -> StdResult<Addr> {
        let maybe_proxy_manager = MANAGER.query(&deps.querier, self.proxy_address(deps)?)?;
        maybe_proxy_manager.ok_or_else(|| StdError::generic_err("proxy admin must be manager."))
    }
    fn os_core(&self, deps: Deps) -> StdResult<Core> {
        Ok(Core {
            manager: self.manager_address(deps)?,
            proxy: self.proxy_address(deps)?,
        })
    }
    /// Get the OS id for the current context.
    fn os_id(&self, deps: Deps) -> StdResult<OsId> {
        OS_ID.query(&deps.querier, self.proxy_address(deps)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_testing::*;
    use speculoos::prelude::*;

    struct MockBinding;

    impl Identification for MockBinding {
        fn proxy_address(&self, _deps: Deps) -> StdResult<Addr> {
            Ok(Addr::unchecked(TEST_PROXY))
        }
    }

    mod core {
        use super::*;
        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn test_proxy_address() {
            let binding = MockBinding;
            let deps = mock_dependencies();

            let res = binding.proxy_address(deps.as_ref());
            assert_that!(res)
                .is_ok()
                .is_equal_to(Addr::unchecked(TEST_PROXY));
        }

        #[test]
        fn test_manager_address() {
            let binding = MockBinding;
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
                .build();

            assert_that!(binding.manager_address(deps.as_ref()))
                .is_ok()
                .is_equal_to(Addr::unchecked(TEST_MANAGER));
        }

        #[test]
        fn test_os_core() {
            let mut deps = mock_dependencies();
            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
                .build();

            let expected_core = Core {
                manager: Addr::unchecked(TEST_MANAGER),
                proxy: Addr::unchecked(TEST_PROXY),
            };

            let binding = MockBinding;
            assert_that!(binding.os_core(deps.as_ref()))
                .is_ok()
                .is_equal_to(expected_core);
        }

        #[test]
        fn os_id() {
            let mut deps = mock_dependencies();
            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, OS_ID, &TEST_OS_ID)
                .build();

            let binding = MockBinding;
            assert_that!(binding.os_id(deps.as_ref()))
                .is_ok()
                .is_equal_to(TEST_OS_ID);
        }
    }
}
