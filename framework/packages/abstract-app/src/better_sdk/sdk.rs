// This is generated

use abstract_sdk::{AbstractSdkError, AbstractSdkResult};
use cosmwasm_std::Addr;

use super::execution_stack::DepsAccess;

use abstract_core::{
    objects::{common_namespace::ADMIN_NAMESPACE, AccountId},
    proxy::state::ACCOUNT_ID,
    version_control::AccountBase,
};
use cw_storage_plus::Item;

// see core::proxy::state::ADMIN

const MANAGER: Item<'_, Option<Addr>> = Item::new(ADMIN_NAMESPACE);

/// Retrieve identifying information about an Account.
/// This includes the manager, proxy, core and account_id.
pub trait AccountIdentification: DepsAccess + Sized {
    /// Get the proxy address for the current account.
    fn proxy_address(&self) -> AbstractSdkResult<Addr>;
    /// Get the manager address for the current account.
    fn manager_address(&self) -> AbstractSdkResult<Addr> {
        let maybe_proxy_manager = MANAGER.query(&self.deps().querier, self.proxy_address()?)?;
        maybe_proxy_manager.ok_or_else(|| AbstractSdkError::AdminNotSet {
            proxy_addr: self.proxy_address().unwrap(),
        })
    }
    /// Get the AccountBase for the current account.
    fn account_base(&self) -> AbstractSdkResult<AccountBase> {
        Ok(AccountBase {
            manager: self.manager_address()?,
            proxy: self.proxy_address()?,
        })
    }
    /// Get the Account id for the current account.
    fn account_id(&self) -> AbstractSdkResult<AccountId> {
        ACCOUNT_ID
            .query(&self.deps().querier, self.proxy_address()?)
            .map_err(Into::into)
    }
}


pub trait SylviaAbstractContract{
    type BaseInstantiateMsg;
    type BaseMigrateMsg;
}


#[cfg(test)]
mod test {
    use super::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::DepsMut;
    use speculoos::prelude::*;

    struct MockBinding<'a> {
        deps: DepsMut<'a>,
    }

    impl<'a> MockBinding<'a> {
        fn new(deps: DepsMut<'a>) -> Self {
            MockBinding { deps }
        }
    }

    impl<'a> DepsAccess for MockBinding<'a> {
        fn deps_mut<'b: 'c, 'c>(&'b mut self) -> cosmwasm_std::DepsMut<'c> {
            self.deps.branch()
        }

        fn deps<'b: 'c, 'c>(&'b self) -> cosmwasm_std::Deps<'c> {
            self.deps.as_ref()
        }
    }

    impl<'a> AccountIdentification for MockBinding<'a> {
        fn proxy_address(&self) -> AbstractSdkResult<Addr> {
            Ok(Addr::unchecked(TEST_PROXY))
        }
    }

    mod account {
        use super::*;
        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn test_proxy_address() {
            let mut deps = mock_dependencies();
            let binding = MockBinding::new(deps.as_mut());

            let res = binding.proxy_address();
            assert_that!(res)
                .is_ok()
                .is_equal_to(Addr::unchecked(TEST_PROXY));
        }

        #[test]
        fn test_manager_address() {
            let mut deps = mock_dependencies();

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
                .build();

            let binding = MockBinding::new(deps.as_mut());
            assert_that!(binding.manager_address())
                .is_ok()
                .is_equal_to(Addr::unchecked(TEST_MANAGER));
        }

        #[test]
        fn test_account() {
            let mut deps = mock_dependencies();
            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
                .build();

            let binding = MockBinding::new(deps.as_mut());
            let expected_account_base = AccountBase {
                manager: Addr::unchecked(TEST_MANAGER),
                proxy: Addr::unchecked(TEST_PROXY),
            };

            assert_that!(binding.account_base())
                .is_ok()
                .is_equal_to(expected_account_base);
        }

        #[test]
        fn account_id() {
            let mut deps = mock_dependencies();
            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding::new(deps.as_mut());
            assert_that!(binding.account_id())
                .is_ok()
                .is_equal_to(TEST_ACCOUNT_ID);
        }
    }
}
