use abstract_core::{
    objects::common_namespace::ADMIN_NAMESPACE, proxy::state::ACCOUNT_ID,
    version_control::AccountBase,
};
use core::objects::AccountId;
use cosmwasm_std::{Addr, Deps};
use cw_storage_plus::Item;

// see core::proxy::state::ADMIN
use crate::{AbstractSdkError, AbstractSdkResult};

const MANAGER: Item<'_, Option<Addr>> = Item::new(ADMIN_NAMESPACE);

/// Retrieve identifying information about an Account.
/// This includes the manager, proxy, core and account_id.
pub trait AccountIdentification: Sized {
    fn proxy_address(&self, deps: Deps) -> AbstractSdkResult<Addr>;
    fn manager_address(&self, deps: Deps) -> AbstractSdkResult<Addr> {
        let maybe_proxy_manager = MANAGER.query(&deps.querier, self.proxy_address(deps)?)?;
        maybe_proxy_manager.ok_or_else(|| AbstractSdkError::AdminNotSet {
            proxy_addr: self.proxy_address(deps).unwrap(),
        })
    }
    fn account_base(&self, deps: Deps) -> AbstractSdkResult<AccountBase> {
        Ok(AccountBase {
            manager: self.manager_address(deps)?,
            proxy: self.proxy_address(deps)?,
        })
    }
    /// Get the Account id for the current context.
    fn account_id(&self, deps: Deps) -> AbstractSdkResult<AccountId> {
        ACCOUNT_ID
            .query(&deps.querier, self.proxy_address(deps)?)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_testing::prelude::*;
    use speculoos::prelude::*;

    struct MockBinding;

    impl AccountIdentification for MockBinding {
        fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
            Ok(Addr::unchecked(TEST_PROXY))
        }
    }

    mod account {
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
        fn test_account() {
            let mut deps = mock_dependencies();
            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
                .build();

            let expected_account_base = AccountBase {
                manager: Addr::unchecked(TEST_MANAGER),
                proxy: Addr::unchecked(TEST_PROXY),
            };

            let binding = MockBinding;
            assert_that!(binding.account_base(deps.as_ref()))
                .is_ok()
                .is_equal_to(expected_account_base);
        }

        #[test]
        fn account_id() {
            let mut deps = mock_dependencies();
            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding;
            assert_that!(binding.account_id(deps.as_ref()))
                .is_ok()
                .is_equal_to(TEST_ACCOUNT_ID);
        }
    }
}
