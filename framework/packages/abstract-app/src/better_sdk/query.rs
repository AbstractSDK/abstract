use abstract_core::{app::BaseExecuteMsg, app::BaseQueryMsg};
use abstract_sdk::{
    feature_objects::AnsHost,
    namespaces::{ADMIN_NAMESPACE, BASE_STATE},
    AbstractSdkResult,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CustomQuery, Deps, DepsMut, Empty, Env, Event, MessageInfo,
};

use super::{
    execution_stack::{CustomEvents, DepsAccess, Executables, ExecutionStack},
    instantiate::AppBaseState,
    nameservice::AbstractNameService,
    sdk::AccountIdentification,
};

pub struct AppQueryCtx<'a, C: CustomQuery = Empty> {
    pub deps: Deps<'a, C>,
    pub env: Env,

    pub base_state: AppBaseState,
}

impl<'a, C: CustomQuery> From<(Deps<'a, C>, Env)> for AppQueryCtx<'a, C> {
    fn from((deps, env): (Deps<'a, C>, Env)) -> Self {
        Self {
            deps,
            env,
            base_state: AppBaseState::default(),
        }
    }
}

#[cw_serde]
pub enum BaseQueryResult {
    Empty,
}

impl BaseQueryResult {
    pub fn generate_response(self) -> AbstractSdkResult<Binary> {
        to_json_binary(&self).map_err(Into::into)
    }
}

impl<'a> AppQueryCtx<'a> {
    pub fn _base(self, msg: BaseQueryMsg) -> AbstractSdkResult<BaseQueryResult> {
        Ok(BaseQueryResult::Empty)
    }
}

impl<'c> DepsAccess for AppQueryCtx<'c> {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b, Empty> {
        unimplemented!()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> cosmwasm_std::Deps<'b, Empty> {
        self.deps
    }
}

impl<'a> AccountIdentification for AppQueryCtx<'a> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(self.base_state.state.load(self.deps.storage)?.proxy_address)
    }
}

impl<'a> AbstractNameService for AppQueryCtx<'a> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(self.base_state.state.load(self.deps.storage)?.ans_host)
    }
}
