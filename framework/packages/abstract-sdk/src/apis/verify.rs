//! # Verification
//! The `Verify` struct provides helper functions that enable the contract to verify if the sender is an Abstract Account, Account admin, etc.
use abstract_std::{
    objects::{registry::RegistryContract, AccountId},
    registry::Account,
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
        # use abstract_testing::prelude::*;
        # let deps = mock_dependencies();
        # let account = admin_account(deps.api);
        # let module = MockModule::new(deps.api, account);

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
    # use abstract_testing::prelude::*;
    # let deps = mock_dependencies();
    # let account = admin_account(deps.api);
    # let module = MockModule::new(deps.api, account);

    let acc_registry: AccountRegistry<MockModule>  = module.account_registry(deps.as_ref()).unwrap();
    ```
*/
#[derive(Clone)]
pub struct AccountRegistry<'a, T: AccountVerification> {
    base: &'a T,
    deps: Deps<'a>,
    vc: RegistryContract,
}

impl<'a, T: AccountVerification> AccountRegistry<'a, T> {
    /// Verify if the provided address is indeed an Abstract Account.
    pub fn assert_account(&self, maybe_account: &Addr) -> AbstractSdkResult<Account> {
        self.vc
            .assert_account(maybe_account, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Get the account base for a given account id.
    pub fn account_base(&self, account_id: &AccountId) -> AbstractSdkResult<Account> {
        self.vc
            .account(account_id, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Get AccountId for given address.
    pub fn account_id(&self, maybe_account_contract_addr: &Addr) -> AbstractSdkResult<AccountId> {
        self.vc
            .account_id(maybe_account_contract_addr, &self.deps.querier)
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
        account::state::ACCOUNT_ID,
        objects::{account::AccountTrace, module::ModuleId, registry::RegistryError},
        registry::state::ACCOUNT_ADDRESSES,
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    struct MockBinding {}

    impl AbstractRegistryAccess for MockBinding {
        fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<RegistryContract> {
            Ok(RegistryContract::new(deps.api)?)
        }
    }

    impl ModuleIdentification for MockBinding {
        fn module_id(&self) -> ModuleId<'static> {
            ModuleId::from("module")
        }
    }

    pub const SECOND_TEST_ACCOUNT_ID: AccountId = AccountId::const_new(2, AccountTrace::Local);

    mod assert_account {

        use super::*;

        #[test]
        fn not_account_fails() {
            let mut deps = mock_dependencies();
            let not_account = Account::new(deps.api.addr_make("not_account"));
            let base = test_account_base(deps.api);

            deps.querier = MockQuerierBuilder::new(deps.api)
                // Setup the addresses as if the Account was registered
                .account(&not_account, TEST_ACCOUNT_ID)
                // update the proxy to be proxy of a different Account
                .account(&base, SECOND_TEST_ACCOUNT_ID)
                .with_contract_item(not_account.addr(), ACCOUNT_ID, &SECOND_TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding {};

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_account(not_account.addr());

            let expected_err = AbstractSdkError::ApiQuery {
                api: AccountRegistry::<MockBinding>::api_id(),
                module_id: binding.module_id().to_owned(),
                error: Box::new(
                    RegistryError::NotAccount(not_account.addr().clone(), SECOND_TEST_ACCOUNT_ID)
                        .into(),
                ),
            };
            assert_eq!(res.unwrap_err(), expected_err);
        }

        #[test]
        fn inactive_account_fails() {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(abstr.account.addr(), ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_key(&abstr.registry, ACCOUNT_ADDRESSES, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding {};

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_account(abstr.account.addr());

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, AbstractSdkError::ApiQuery { .. }))
                .matches(|e| {
                    e.to_string().contains(
                        &RegistryError::UnknownAccountId {
                            account_id: TEST_ACCOUNT_ID,
                            registry_addr: abstr.registry.clone(),
                        }
                        .to_string(),
                    )
                });
        }

        #[test]
        fn returns_account() {
            let mut deps = mock_dependencies();
            let account = test_account_base(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(account.addr(), ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    &abstr.registry,
                    ACCOUNT_ADDRESSES,
                    (&TEST_ACCOUNT_ID, account.clone()),
                )
                .build();

            let binding = MockBinding {};

            let res = binding
                .account_registry(deps.as_ref())
                .unwrap()
                .assert_account(account.addr());

            assert_that!(res).is_ok().is_equal_to(account);
        }
    }
}
