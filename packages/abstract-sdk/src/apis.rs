pub mod bank;
pub mod dex;
pub mod execution;
pub mod ibc;
pub mod modules;
pub mod vault;
pub mod verify;
pub mod version_register;

pub(crate) use crate::base::features::*;

#[cfg(test)]
mod test_common {
    use crate::apis::{AbstractNameService, Identification};
    pub use abstract_testing::mock_module::*;
    pub use abstract_testing::*;
    pub use cosmwasm_std::testing::*;
    pub use cosmwasm_std::*;
    use os::objects::ans_host::AnsHost;
    pub use speculoos::prelude::*;

    // We implement the following traits here for the mock module (in this package) to avoid a circular dependency
    impl Identification for MockModule {
        fn proxy_address(&self, _deps: Deps) -> Result<Addr, StdError> {
            Ok(Addr::unchecked(TEST_PROXY))
        }
    }

    impl AbstractNameService for MockModule {
        fn ans_host(&self, _deps: Deps) -> StdResult<AnsHost> {
            Ok(AnsHost {
                address: Addr::unchecked("ans"),
            })
        }
    }
}
