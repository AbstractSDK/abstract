use abstract_std::{account::state::ACCOUNT_ID, registry::Account};
use cosmwasm_std::Deps;

use crate::std::objects::AccountId;
// see std::proxy::state::ADMIN
use crate::AbstractSdkResult;

/// Retrieve identifying information about an Account.
/// This includes the manager, proxy, core and account_id.
pub trait AccountIdentification: Sized {
    /// Get the account address
    fn account(&self, deps: Deps) -> AbstractSdkResult<Account>;

    /// Get the Account id for the current account.
    fn account_id(&self, deps: Deps) -> AbstractSdkResult<AccountId> {
        ACCOUNT_ID
            .query(&deps.querier, self.account(deps)?.into_addr())
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::MockApi;
    use speculoos::prelude::*;

    use super::*;

    struct MockBinding {
        mock_api: MockApi,
    }

    impl AccountIdentification for MockBinding {
        fn account(&self, _deps: Deps) -> AbstractSdkResult<Account> {
            let account = test_account(self.mock_api);
            Ok(account)
        }
    }

    mod account {
        use cosmwasm_std::testing::mock_dependencies;

        use super::*;

        #[test]
        fn test_account_address() {
            let deps = mock_dependencies();
            let binding = MockBinding { mock_api: deps.api };

            let account = test_account(deps.api);

            let res = binding.account(deps.as_ref());
            assert_that!(res).is_ok().is_equal_to(account);
        }

        #[test]
        fn account_id() {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(account.addr(), ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding { mock_api: deps.api };
            assert_that!(binding.account_id(deps.as_ref()))
                .is_ok()
                .is_equal_to(TEST_ACCOUNT_ID);
        }
    }
}
