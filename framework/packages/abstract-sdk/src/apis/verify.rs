//! # Verification
//! The `Verify` struct provides helper functions that enable the contract to verify if the sender is an Abstract Account, Account admin, etc.
use abstract_std::{
    objects::ownership::nested_admin::assert_account_calling_to_as_admin_is_self,
    objects::{registry::RegistryContract, AccountId},
    registry::Account,
};
use cosmwasm_std::{Addr, Deps, Env};

use super::AbstractApi;
use crate::{
    cw_helpers::ApiQuery,
    features::{AbstractRegistryAccess, ModuleIdentification},
    AbstractSdkError, AbstractSdkResult,
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
    fn account_registry<'a>(
        &'a self,
        deps: Deps<'a>,
        env: &Env,
    ) -> AbstractSdkResult<AccountRegistry<Self>> {
        let vc = self.abstract_registry(deps, env)?;
        Ok(AccountRegistry {
            base: self,
            deps,
            registry: vc,
        })
    }
}

impl<T> AccountVerification for T where T: AbstractRegistryAccess + ModuleIdentification {}

impl<'a, T: AccountVerification> AbstractApi<T> for AccountRegistry<'a, T> {
    const API_ID: &'static str = "AccountRegistry";

    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
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
    registry: RegistryContract,
}

impl<'a, T: AccountVerification> AccountRegistry<'a, T> {
    /// Verify if the provided address is indeed an Abstract Account.
    pub fn assert_is_account(&self, maybe_account: &Addr) -> AbstractSdkResult<Account> {
        self.registry
            .assert_account(maybe_account, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Get the account for a given account id.
    pub fn account(&self, account_id: &AccountId) -> AbstractSdkResult<Account> {
        self.registry
            .account(account_id, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Get AccountId for given address.
    pub fn account_id(&self, maybe_account_contract_addr: &Addr) -> AbstractSdkResult<AccountId> {
        self.registry
            .account_id(maybe_account_contract_addr, &self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Get namespace registration fee
    pub fn namespace_registration_fee(&self) -> AbstractSdkResult<Option<cosmwasm_std::Coin>> {
        self.registry
            .namespace_registration_fee(&self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Verify if the provided address is indeed an Abstract Account AND if the current execution has admin rights.
    pub fn assert_is_account_admin(
        &self,
        env: &Env,
        maybe_account: &Addr,
    ) -> AbstractSdkResult<Account> {
        let account = self.assert_is_account(maybe_account)?;

        if !assert_account_calling_to_as_admin_is_self(&self.deps.querier, env, maybe_account) {
            return Err(AbstractSdkError::OnlyAdmin {});
        }
        Ok(account)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::{apis::traits::test::abstract_api_test, AbstractSdkError};
    use abstract_std::{
        account::state::ACCOUNT_ID,
        objects::{account::AccountTrace, module::ModuleId, registry::RegistryError},
        registry::{self, state::ACCOUNT_ADDRESSES},
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Coin};

    struct MockBinding {}

    impl AbstractRegistryAccess for MockBinding {
        fn abstract_registry(&self, deps: Deps, env: &Env) -> AbstractSdkResult<RegistryContract> {
            RegistryContract::new(deps.api, env).map_err(Into::into)
        }
    }

    impl ModuleIdentification for MockBinding {
        fn module_id(&self) -> ModuleId<'static> {
            ModuleId::from(TEST_MODULE_ID)
        }
    }

    pub const SECOND_TEST_ACCOUNT_ID: AccountId = AccountId::const_new(2, AccountTrace::Local);

    mod assert_account {

        use super::*;

        #[coverage_helper::test]
        fn not_account_fails() {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let not_account = Account::new(deps.api.addr_make("not_account"));
            let base = test_account(deps.api);

            deps.querier = MockQuerierBuilder::new(deps.api)
                // Setup the addresses as if the Account was registered
                .account(&not_account, TEST_ACCOUNT_ID)
                // update the account to be account of a different Account
                .account(&base, SECOND_TEST_ACCOUNT_ID)
                .with_contract_item(not_account.addr(), ACCOUNT_ID, &SECOND_TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding {};

            let res = binding
                .account_registry(deps.as_ref(), &env)
                .unwrap()
                .assert_is_account(not_account.addr());

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

        #[coverage_helper::test]
        fn inactive_account_fails() {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(abstr.account.addr(), ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_key(&abstr.registry, ACCOUNT_ADDRESSES, &TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding {};

            let res = binding
                .account_registry(deps.as_ref(), &env)
                .unwrap()
                .assert_is_account(abstr.account.addr());

            let expected_err = AbstractSdkError::ApiQuery {
                api: AccountRegistry::<MockBinding>::api_id(),
                module_id: binding.module_id().to_owned(),
                error: Box::new(
                    RegistryError::UnknownAccountId {
                        account_id: TEST_ACCOUNT_ID,
                        registry_addr: abstr.registry,
                    }
                    .into(),
                ),
            };
            assert_eq!(res.unwrap_err(), expected_err);
        }

        #[coverage_helper::test]
        fn returns_account() {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let account = test_account(deps.api);

            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();

            let binding = MockBinding {};

            let registry = binding.account_registry(deps.as_ref(), &env).unwrap();
            let res = registry.assert_is_account(account.addr());

            assert_eq!(res, Ok(account.clone()));

            let account_id = registry.account_id(account.addr());
            assert_eq!(account_id, Ok(TEST_ACCOUNT_ID));
        }
    }

    #[coverage_helper::test]
    fn namespace_fee() {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);

        deps.querier = abstract_mock_querier(deps.api);

        let binding = MockBinding {};

        // Namespace registration fee query
        {
            let registry = binding.account_registry(deps.as_ref(), &env).unwrap();
            let res = registry.namespace_registration_fee();

            assert_eq!(res, Ok(None));
        }

        let abstr = AbstractMockAddrs::new(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .with_contract_item(
                &abstr.registry,
                registry::state::CONFIG,
                &registry::Config {
                    security_disabled: true,
                    namespace_registration_fee: Some(Coin::new(42_u128, "foo")),
                },
            )
            .build();

        let registry = binding.account_registry(deps.as_ref(), &env).unwrap();
        let res = registry.namespace_registration_fee();

        assert_eq!(res, Ok(Some(Coin::new(42_u128, "foo"))));
    }

    #[coverage_helper::test]
    fn abstract_api() {
        let deps = mock_dependencies();
        let module = MockBinding {};
        let env = mock_env_validated(deps.api);

        let account_registry = module.account_registry(deps.as_ref(), &env).unwrap();

        abstract_api_test(account_registry);
    }
}
