//! # Structs that implement a feature trait
//!
//! Feature objects are objects that store sufficient data to unlock some functionality.
//! These objects are mostly used internally to easy re-use application code without
//! requiring the usage of a base contract.

pub use abstract_std::objects::{ans_host::AnsHost, version_control::VersionControlContract};
use abstract_std::{version_control::Account, VERSION_CONTROL};
use cosmwasm_std::{Addr, Deps};

use crate::{
    features::{AccountIdentification, ModuleIdentification},
    std::ACCOUNT,
    AbstractSdkResult,
};


impl AccountIdentification for Account {
    fn account(&self, _deps: Deps) -> AbstractSdkResult<Account> {
        Ok(self.clone())
    }
}

impl ModuleIdentification for Account {
    /// Any actions executed by the core will be by the proxy address
    fn module_id(&self) -> &'static str {
        ACCOUNT
    }
}

impl crate::features::AbstractRegistryAccess for VersionControlContract {
    fn abstract_registry(&self, _deps: Deps) -> AbstractSdkResult<VersionControlContract> {
        Ok(self.clone())
    }
}

impl ModuleIdentification for VersionControlContract {
    fn module_id(&self) -> abstract_std::objects::module::ModuleId<'static> {
        VERSION_CONTROL
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
            let deps = mock_dependencies();
            let account_base = test_account_base(deps.api);
            let address = account_base.proxy;
            let proxy = ProxyContract::new(address.clone());

            assert_that!(proxy.proxy_address(deps.as_ref()))
                .is_ok()
                .is_equal_to(address);
        }

        #[test]
        fn should_identify_self_as_abstract_proxy() {
            let proxy = ProxyContract::new(Addr::unchecked("test"));

            assert_that!(proxy.module_id()).is_equal_to(ACCOUNT);
        }
    }

    mod base {
        use cosmwasm_std::testing::{mock_dependencies, MockApi};

        use super::*;

        #[test]
        fn test_account() {
            let deps = mock_dependencies();
            let account_base = test_account_base(deps.api);

            assert_that!(account_base.account_base(deps.as_ref()))
                .is_ok()
                .is_equal_to(account_base);
        }

        #[test]
        fn should_identify_self_as_abstract_proxy() {
            let account_base = test_account_base(MockApi::default());

            assert_that!(account_base.module_id()).is_equal_to(ACCOUNT);
        }
    }
}
