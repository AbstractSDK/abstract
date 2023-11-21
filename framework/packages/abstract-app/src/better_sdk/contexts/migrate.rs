use abstract_sdk::{
    feature_objects::AnsHost,
    AbstractSdkResult,
};
use cosmwasm_std::{
    Addr, CustomQuery, DepsMut, Empty, Env, Event, Attribute, Binary, Response
};

use crate::AppError;

use crate::better_sdk::{
    account_identification::AccountIdentification,
    execution_stack::{CustomEvents, DepsAccess, Executables, ExecutionStack, CustomData, ResponseGenerator},
    nameservice::AbstractNameService, sdk::BASE_STATE,
};

pub struct AppMigrateCtx<'a, C: CustomQuery = Empty> {
    pub deps: DepsMut<'a, C>,
    pub env: Env,

    pub executables: Executables,
    pub events: Vec<Event>,
    pub attributes: Vec<Attribute>,
    pub data: Option<Binary>,
}

impl<'a, C: CustomQuery> From<(DepsMut<'a, C>, Env)> for AppMigrateCtx<'a, C> {

    fn from(
        (deps, env): (DepsMut<'a, C>, Env),
    ) -> Self {
        Self {
            deps,
            env,
            executables: Executables::default(),
            events: vec![],
            attributes: vec![],
            data: None
        }
    }
}

impl<'c> DepsAccess for AppMigrateCtx<'c, Empty> {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b, Empty> {
        self.deps.branch()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> cosmwasm_std::Deps<'b, Empty> {
        self.deps.as_ref()
    }
}

impl<'a> CustomEvents for AppMigrateCtx<'a> {
    fn add_event(&mut self, event_name: &str, attributes: Vec<(&str, &str)>) {
        self.events
            .push(Event::new(event_name).add_attributes(attributes))
    }
    fn events(&self) -> Vec<Event> {
        self.events.clone()
    }

    fn add_attributes(&mut self,attributes: Vec<(&str, &str)>) {
        self.attributes.extend(attributes.into_iter().map(|(key, value)| Attribute::new(key, value)))
    }

    fn attributes(&self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}
impl<'a> CustomData for AppMigrateCtx<'a> {
    fn set_data(&mut self, data: impl Into<Binary>){
        self.data = Some(data.into());
    }  
    fn data(&self) -> Option<Binary> {
        self.data.clone()
    }
}

impl<'a> ExecutionStack for AppMigrateCtx<'a> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.executables
    }
}
impl TryInto<Response<Empty>> for AppMigrateCtx<'_>{
    type Error = AppError;
    fn try_into(mut self) -> Result<Response<Empty>, Self::Error> {
        Ok(self._generate_response()?)
    }
}

impl<'a> AccountIdentification for AppMigrateCtx<'a> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(BASE_STATE.load(self.deps.storage)?.proxy_address)
    }
}
impl<'a> AbstractNameService for AppMigrateCtx<'a> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(BASE_STATE.load(self.deps.storage)?.ans_host)
    }
}

