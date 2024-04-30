//! # Structs that implement a feature trait
//!
//! Feature objects are objects that store sufficient data to unlock some functionality.
//! These objects are mostly used internally to easy re-use application code without
//! requiring the usage of a base contract.

pub use abstract_std::objects::{ans_host::AnsHost, version_control::VersionControlContract};
use abstract_std::version_control::AccountBase;
use cosmwasm_std::{Addr, Deps};

use crate::{
    features::{AccountIdentification, ModuleIdentification},
    std::PROXY,
    AbstractSdkResult,
};

/// Store a proxy contract address.
/// Implements [`AccountIdentification`].
#[derive(Clone)]
pub struct ProxyContract {
    /// Address of the proxy contract
    pub contract_address: Addr,
}

impl ProxyContract {
    /// Construct a new proxy contract feature object.
    pub fn new(address: Addr) -> Self {
        Self {
            contract_address: address,
        }
    }
}

impl AccountIdentification for ProxyContract {
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.contract_address.clone())
    }
}

impl ModuleIdentification for ProxyContract {
    fn module_id(&self) -> &'static str {
        PROXY
    }
}

impl AccountIdentification for AccountBase {
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.proxy.clone())
    }

    fn manager_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.manager.clone())
    }

    fn account_base(&self, _deps: Deps) -> AbstractSdkResult<AccountBase> {
        Ok(self.clone())
    }
}

impl ModuleIdentification for AccountBase {
    /// Any actions executed by the core will be by the proxy address
    fn module_id(&self) -> &'static str {
        PROXY
    }
}

impl crate::features::AbstractRegistryAccess for VersionControlContract {
    fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<VersionControlContract> {
        Ok(self.clone())
    }
}

impl crate::features::AbstractNameService for AnsHost {
    fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use abstract_testing::prelude::*;
    use speculoos::prelude::*;

    use super::*;

    mod version_control {
        use cosmwasm_std::testing::mock_dependencies;

        use super::*;
        use crate::features::AbstractRegistryAccess;

        #[test]
        fn test_registry() {
            let address = Addr::unchecked("version");
            let vc = VersionControlContract::new(address.clone());

            let deps = mock_dependencies();

            assert_that!(vc.abstract_registry(deps.as_ref()))
                .is_ok()
                .is_equal_to(vc);
        }
    }

    mod proxy {
        use cosmwasm_std::testing::mock_dependencies;

        use super::*;

        #[test]
        fn test_proxy_address() {
            let address = Addr::unchecked(TEST_PROXY);
            let proxy = ProxyContract::new(address.clone());
            let deps = mock_dependencies();

            assert_that!(proxy.proxy_address(deps.as_ref()))
                .is_ok()
                .is_equal_to(address);
        }

        #[test]
        fn should_identify_self_as_abstract_proxy() {
            let proxy = ProxyContract::new(Addr::unchecked(TEST_PROXY));

            assert_that!(proxy.module_id()).is_equal_to(PROXY);
        }
    }

    mod base {
        use cosmwasm_std::testing::mock_dependencies;

        use super::*;

        fn test_account_base() -> AccountBase {
            AccountBase {
                manager: Addr::unchecked(TEST_MANAGER),
                proxy: Addr::unchecked(TEST_PROXY),
            }
        }

        #[test]
        fn test_proxy_address() {
            let address = Addr::unchecked(TEST_PROXY);
            let account_base = test_account_base();

            let deps = mock_dependencies();

            assert_that!(account_base.proxy_address(deps.as_ref()))
                .is_ok()
                .is_equal_to(address);
        }

        #[test]
        fn test_manager_address() {
            let manager_addrsess = Addr::unchecked(TEST_MANAGER);
            let account_base = test_account_base();

            let deps = mock_dependencies();

            assert_that!(account_base.manager_address(deps.as_ref()))
                .is_ok()
                .is_equal_to(manager_addrsess);
        }

        #[test]
        fn test_account() {
            let account_base = test_account_base();

            let deps = mock_dependencies();

            assert_that!(account_base.account_base(deps.as_ref()))
                .is_ok()
                .is_equal_to(account_base);
        }

        #[test]
        fn should_identify_self_as_abstract_proxy() {
            let account_base = test_account_base();

            assert_that!(account_base.module_id()).is_equal_to(PROXY);
        }
    }
}
