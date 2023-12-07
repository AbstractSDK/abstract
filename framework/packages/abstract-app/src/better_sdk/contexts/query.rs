use abstract_sdk::{feature_objects::AnsHost, AbstractSdkResult};
use cosmwasm_std::{Addr, CustomQuery, Deps, DepsMut, Empty, Env, MessageInfo};

use crate::better_sdk::{
    account_identification::AccountIdentification, execution_stack::DepsAccess,
    nameservice::AbstractNameService, sdk::BASE_STATE,
};

pub struct AppQueryCtx<'a, C: CustomQuery = Empty> {
    pub deps: Deps<'a, C>,
    pub env: Env,
}

impl<'a, C: CustomQuery> From<(Deps<'a, C>, Env)> for AppQueryCtx<'a, C> {
    fn from((deps, env): (Deps<'a, C>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'c> DepsAccess for AppQueryCtx<'c> {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b, Empty> {
        unimplemented!()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> cosmwasm_std::Deps<'b, Empty> {
        self.deps
    }

    fn env(&self) -> Env {
        self.env.clone()
    }

    fn message_info(&self) -> MessageInfo {
        unimplemented!()
    }
}

impl<'a> AccountIdentification for AppQueryCtx<'a> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(BASE_STATE.load(self.deps.storage)?.proxy_address)
    }
}

impl<'a> AbstractNameService for AppQueryCtx<'a> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(BASE_STATE.load(self.deps.storage)?.ans_host)
    }
}
