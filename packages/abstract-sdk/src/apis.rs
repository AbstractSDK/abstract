pub mod bank;
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
    pub use cosmwasm_std::testing::*;
    pub use cosmwasm_std::*;
    use os::objects::ans_host::AnsHost;
    use os::{api, app};
    pub use speculoos::prelude::*;

    /// A mock module that can be used for testing.
    /// @dev This is a copy of the mock module from the abstract-testing crate. It is copied here to
    /// avoid a circular dependency.
    pub struct MockModule {}

    impl MockModule {
        pub const fn new() -> Self {
            Self {}
        }
    }

    pub const TEST_PROXY: &str = "proxy_address";
    pub const TEST_MANAGER: &str = "manager_address";

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

    #[cosmwasm_schema::cw_serde]
    pub struct MockModuleExecuteMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockModuleQueryMsg {}

    impl api::ApiExecuteMsg for MockModuleExecuteMsg {}

    impl api::ApiQueryMsg for MockModuleQueryMsg {}

    impl app::AppExecuteMsg for MockModuleExecuteMsg {}

    impl app::AppQueryMsg for MockModuleQueryMsg {}
}
