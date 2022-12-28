use abstract_os::objects::ans_host::AnsHost;
use abstract_os::{api, app};
use cosmwasm_std::{Addr, Deps, StdError, StdResult};

#[cfg(feature = "sdk")]
use abstract_sdk::base::features::{AbstractNameService, Identification};

pub struct MockModule {}

impl MockModule {
    pub const fn new() -> Self {
        Self {}
    }
}

pub const TEST_PROXY: &str = "proxy_address";
pub const TEST_MANAGER: &str = "manager_address";

#[cfg(feature = "sdk")]
impl Identification for MockModule {
    fn proxy_address(&self, _deps: Deps) -> Result<Addr, StdError> {
        Ok(Addr::unchecked(TEST_PROXY))
    }
}

#[cfg(feature = "sdk")]
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
