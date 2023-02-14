//! # Feature Objects
//! Feature objects are objects that store sufficient data to unlock a set of APIs.
//! These objects are mostly used internally to easy re-use application code without
//! requiring the usage of a base contract.  

use abstract_os::version_control::Core;
use cosmwasm_std::{Addr, Deps};

use crate::apis::ModuleIdentification;
use crate::base::features::{AbstractRegistryAccess, Identification};
use crate::AbstractSdkResult;
pub use abstract_os::objects::ans_host::AnsHost;
use os::PROXY;

/// Store the Version Control contract.
/// Implements [`AbstractRegistryAccess`]
#[derive(Clone)]
pub struct VersionControlContract {
    pub address: Addr,
}

impl VersionControlContract {
    pub fn new(address: Addr) -> Self {
        Self { address }
    }
}

impl AbstractRegistryAccess for VersionControlContract {
    fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.address.clone())
    }
}

/// Store a proxy contract address.
/// Implements [`Identification`].
#[derive(Clone)]
pub struct ProxyContract {
    pub contract_address: Addr,
}

impl ProxyContract {
    pub fn new(address: Addr) -> Self {
        Self {
            contract_address: address,
        }
    }
}

impl Identification for ProxyContract {
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.contract_address.clone())
    }
}

impl ModuleIdentification for ProxyContract {
    fn module_id(&self) -> &'static str {
        PROXY
    }
}

impl Identification for Core {
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.proxy.clone())
    }

    fn manager_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.manager.clone())
    }

    fn os_core(&self, _deps: Deps) -> AbstractSdkResult<Core> {
        Ok(self.clone())
    }
}

impl ModuleIdentification for Core {
    /// Any actions executed by the core will be by the proxy address
    fn module_id(&self) -> &'static str {
        PROXY
    }
}

impl crate::base::features::AbstractNameService for AnsHost {
    fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<abstract_os::objects::ans_host::AnsHost> {
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abstract_testing::{TEST_MANAGER, TEST_PROXY};
    use speculoos::prelude::*;

    mod version_control {
        use super::*;
        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn test_registry() {
            let address = Addr::unchecked("version");
            let vc = VersionControlContract::new(address.clone());

            let deps = mock_dependencies();

            assert_that!(vc.abstract_registry(deps.as_ref()))
                .is_ok()
                .is_equal_to(address);
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

    mod core {
        use super::*;
        use cosmwasm_std::testing::mock_dependencies;

        fn test_core() -> Core {
            Core {
                manager: Addr::unchecked(TEST_MANAGER),
                proxy: Addr::unchecked(TEST_PROXY),
            }
        }

        #[test]
        fn test_proxy_address() {
            let address = Addr::unchecked(TEST_PROXY);
            let core = test_core();

            let deps = mock_dependencies();

            assert_that!(core.proxy_address(deps.as_ref()))
                .is_ok()
                .is_equal_to(address);
        }

        #[test]
        fn test_manager_address() {
            let manager_addrsess = Addr::unchecked(TEST_MANAGER);
            let core = test_core();

            let deps = mock_dependencies();

            assert_that!(core.manager_address(deps.as_ref()))
                .is_ok()
                .is_equal_to(manager_addrsess);
        }

        #[test]
        fn test_os_core() {
            let core = test_core();

            let deps = mock_dependencies();

            assert_that!(core.os_core(deps.as_ref()))
                .is_ok()
                .is_equal_to(core);
        }

        #[test]
        fn should_identify_self_as_abstract_proxy() {
            let core = test_core();

            assert_that!(core.module_id()).is_equal_to(PROXY);
        }
    }
}
