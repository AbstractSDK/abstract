use crate::{core::objects::AccountId, AccountAction};
use abstract_core::{
    objects::common_namespace::ADMIN_NAMESPACE, proxy::state::ACCOUNT_ID,
    version_control::AccountBase,
};
use cosmwasm_std::{Addr, Deps, CosmosMsg, DepsMut, Api, Env};
use cw_storage_plus::Item;
use crate::{AbstractSdkError, AbstractSdkResult};


pub trait DepsAccess<'a:'b, 'b:'c, 'c> {
    fn deps_mut(&'b mut self) -> DepsMut<'c>;
    fn deps(&'b self) -> Deps<'c>;

    fn api(&'b self) -> &'c dyn Api {
        self.deps().api
    }
}

#[derive(Clone)]
pub enum Executable {
    CosmosMsg(CosmosMsg),
    AccountAction(AccountAction),
}
/// A list of messages that can be executed
/// Can only be appended to and iterated over.
pub struct Executables(pub(crate) Vec<Executable>);

impl Default for Executables {
    fn default() -> Self {
        Self(Vec::with_capacity(8))
    }
}

impl Executables {
    pub fn push(&mut self, msg: Executable) {
        self.0.push(msg)
    }
}

pub trait ExecutionStack: Sized {
    fn stack_mut(&mut self) -> &mut Executables;
    /// Get the manager address for the current account.
    fn push_executable(&mut self, executable: Executable) {
        self.stack_mut().push(executable);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use abstract_testing::prelude::*;
    use speculoos::prelude::*;

    struct MockBinding;

    // impl AccountIdentification for MockBinding {
    //     fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
    //         Ok(Addr::unchecked(TEST_PROXY))
    //     }
    // }

    // mod account {
    //     use super::*;
    //     use cosmwasm_std::testing::mock_dependencies;

    //     #[test]
    //     fn test_proxy_address() {
    //         let binding = MockBinding;
    //         let deps = mock_dependencies();

    //         let res = binding.proxy_address(deps.as_ref());
    //         assert_that!(res)
    //             .is_ok()
    //             .is_equal_to(Addr::unchecked(TEST_PROXY));
    //     }

    //     #[test]
    //     fn test_manager_address() {
    //         let binding = MockBinding;
    //         let mut deps = mock_dependencies();

    //         deps.querier = MockQuerierBuilder::default()
    //             .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
    //             .build();

    //         assert_that!(binding.manager_address(deps.as_ref()))
    //             .is_ok()
    //             .is_equal_to(Addr::unchecked(TEST_MANAGER));
    //     }

    //     #[test]
    //     fn test_account() {
    //         let mut deps = mock_dependencies();
    //         deps.querier = MockQuerierBuilder::default()
    //             .with_contract_item(TEST_PROXY, MANAGER, &Some(Addr::unchecked(TEST_MANAGER)))
    //             .build();

    //         let expected_account_base = AccountBase {
    //             manager: Addr::unchecked(TEST_MANAGER),
    //             proxy: Addr::unchecked(TEST_PROXY),
    //         };

    //         let binding = MockBinding;
    //         assert_that!(binding.account_base(deps.as_ref()))
    //             .is_ok()
    //             .is_equal_to(expected_account_base);
    //     }

    //     #[test]
    //     fn account_id() {
    //         let mut deps = mock_dependencies();
    //         deps.querier = MockQuerierBuilder::default()
    //             .with_contract_item(TEST_PROXY, ACCOUNT_ID, &TEST_ACCOUNT_ID)
    //             .build();

    //         let binding = MockBinding;
    //         assert_that!(binding.account_id(deps.as_ref()))
    //             .is_ok()
    //             .is_equal_to(TEST_ACCOUNT_ID);
    //     }
    // }
}
