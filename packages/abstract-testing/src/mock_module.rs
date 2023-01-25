use abstract_os::{api, app};
use cosmwasm_std::{Deps, StdError, StdResult};

use crate::{TEST_ANS_HOST, TEST_PROXY};
#[cfg(feature = "sdk")]
use ::{
    abstract_os::objects::ans_host::AnsHost,
    abstract_sdk::base::features::{AbstractNameService, Identification, ModuleIdentification},
    cosmwasm_std::Addr,
};

pub struct MockModule {}

impl MockModule {
    pub const fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "sdk")]
impl Identification for MockModule {
    fn proxy_address(&self, _deps: Deps) -> Result<Addr, StdError> {
        Ok(Addr::unchecked(TEST_PROXY))
    }
}

#[cfg(feature = "sdk")]
impl ModuleIdentification for MockModule {
    fn module_id(&self) -> &'static str {
        "mock_module"
    }
}

#[cfg(feature = "sdk")]
impl AbstractNameService for MockModule {
    fn ans_host(&self, _deps: Deps) -> StdResult<AnsHost> {
        Ok(AnsHost {
            address: Addr::unchecked(TEST_ANS_HOST),
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
