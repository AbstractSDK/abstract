use abstract_core::app::BaseExecuteMsg;
use abstract_sdk::{
    feature_objects::AnsHost,
    namespaces::{ADMIN_NAMESPACE, BASE_STATE},
    AbstractSdkResult,
};
use cosmwasm_std::{Addr, CustomQuery, DepsMut, Empty, Env, Event, MessageInfo};

use super::{
    execution_stack::{CustomEvents, DepsAccess, Executables, ExecutionStack},
    instantiate::AppBaseState,
    nameservice::AbstractNameService,
    sdk::AccountIdentification,
};

pub struct AppExecCtx<'a, C: CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub info: MessageInfo,

    pub base_state: AppBaseState,
    pub events: Vec<Event>,
    pub executables: Executables,
}

impl<'a, C: CustomQuery> From<(DepsMut<'a, C>, Env, MessageInfo)> for AppExecCtx<'a, C> {
    fn from((deps, env, info): (DepsMut<'a, C>, Env, MessageInfo)) -> Self {
        Self {
            deps,
            env,
            info,
            base_state: AppBaseState::default(),
            events: vec![],
            executables: Executables::default(),
        }
    }
}

impl<'a> AppExecCtx<'a> {
    pub fn _base(self, msg: BaseExecuteMsg) -> AbstractSdkResult<Self> {
        // We need to port this implementation from the current app definition
        todo!();
        Ok(self)
    }
}

impl<'c> DepsAccess for AppExecCtx<'c> {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b, Empty> {
        self.deps.branch()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> cosmwasm_std::Deps<'b, Empty> {
        self.deps.as_ref()
    }
}

impl<'a> CustomEvents for AppExecCtx<'a> {
    fn add_event(&mut self, event_name: &str, attributes: Vec<(&str, &str)>) {
        self.events
            .push(Event::new(event_name).add_attributes(attributes))
    }
    fn events(&self) -> Vec<Event> {
        self.events.clone()
    }
}

impl<'a> ExecutionStack for AppExecCtx<'a> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.executables
    }
}

impl<'a> AccountIdentification for AppExecCtx<'a> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(self.base_state.state.load(self.deps.storage)?.proxy_address)
    }
}

impl<'a> AbstractNameService for AppExecCtx<'a> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(self.base_state.state.load(self.deps.storage)?.ans_host)
    }
}
