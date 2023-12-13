//! # Structs that implement a feature trait
//!
//! Feature objects are objects that store sufficient data to unlock some functionality.
//! These objects are mostly used internally to easy re-use application code without
//! requiring the usage of a base contract.

use crate::core::PROXY;
use crate::features::{AbstractRegistryAccess, AccountIdentification, DepsAccess};
use crate::{features::ModuleIdentification, AbstractSdkResult};
pub use abstract_core::objects::{ans_host::AnsHost, version_control::VersionControlContract};
use abstract_core::version_control::AccountBase;
use cosmwasm_std::{Addr, Deps};

/// Temporary features struct to be able to use those traits inside contracts
pub struct Feature<'a, T> {
    contract: &'a T,
    deps: Deps<'a>,
}

impl<'a, T> Feature<'a, T> {
    /// Creates a feature from a core struct to query abstract information on it directly
    pub fn from_contract(contract: &'a T, deps: Deps<'a>) -> Self {
        Self { contract, deps }
    }
}

impl<'a, T: AbstractRegistryAccess> AbstractRegistryAccess for Feature<'a, T> {
    fn abstract_registry(&self) -> AbstractSdkResult<VersionControlContract> {
        self.contract.abstract_registry()
    }
}

impl<'a, T: ModuleIdentification> ModuleIdentification for Feature<'a, T> {
    fn module_id(&self) -> &str {
        self.contract.module_id()
    }
}

impl<'a> AccountIdentification for Feature<'a, ProxyContract> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(self.contract.contract_address.clone())
    }
}
impl<'a> AccountIdentification for Feature<'a, AccountBase> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(self.contract.proxy.clone())
    }
}

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

impl ModuleIdentification for ProxyContract {
    fn module_id(&self) -> &str {
        PROXY
    }
}
impl ModuleIdentification for AccountBase {
    /// Any actions executed by the core will be by the proxy address
    fn module_id(&self) -> &str {
        PROXY
    }
}

impl AbstractRegistryAccess for VersionControlContract {
    fn abstract_registry(&self) -> AbstractSdkResult<VersionControlContract> {
        Ok(self.clone())
    }
}

impl<'m, T> DepsAccess for Feature<'m, T> {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> cosmwasm_std::DepsMut<'b> {
        unimplemented!()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> Deps<'b> {
        self.deps
    }

    fn env(&self) -> cosmwasm_std::Env {
        unimplemented!()
    }

    fn message_info(&self) -> cosmwasm_std::MessageInfo {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abstract_testing::prelude::*;
    use speculoos::prelude::*;

    mod version_control {
        use super::*;

        use crate::features::AbstractRegistryAccess;

        #[test]
        fn test_registry() {
            let address = Addr::unchecked("version");
            let vc = VersionControlContract::new(address.clone());

            assert_that!(vc.abstract_registry()).is_ok().is_equal_to(vc);
        }
    }

    mod proxy {
        use super::*;
        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn test_proxy_address() {
            let address = Addr::unchecked(TEST_PROXY);
            let proxy = ProxyContract::new(address.clone());
            let deps = mock_dependencies();

            assert_that!(Feature::from_contract(&proxy, deps.as_ref()).proxy_address())
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

            assert_that!(account_base.proxy_address()).is_equal_to(address);
        }

        #[test]
        fn test_manager_address() {
            let manager_addrsess = Addr::unchecked(TEST_MANAGER);
            let account_base = test_account_base();

            assert_that!(account_base.manager_address()).is_equal_to(manager_addrsess);
        }

        #[test]
        fn test_account() {
            let account_base = test_account_base();

            assert_that!(account_base.account_base()).is_equal_to(account_base);
        }

        #[test]
        fn should_identify_self_as_abstract_proxy() {
            let account_base = test_account_base();

            assert_that!(account_base.module_id()).is_equal_to(PROXY);
        }
    }
}
