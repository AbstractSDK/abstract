use abstract_sdk::{feature_objects::AnsHost, AbstractSdkResult, base::Handler, AbstractSdkError};
use cosmwasm_std::{Addr, Deps, DepsMut, Empty, Env, Event, MessageInfo, Response};

use super::{
    execution_stack::{CustomEvents, DepsAccess, Executables, ExecutionStack},
    nameservice::AbstractNameService,
    sdk::AccountIdentification,
    TestContract,
};

// This is macro generated (because it has the struct in its types)
impl<'a, Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> AccountIdentification for TestContract<'a, Module, Error> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(Addr::unchecked("proxy_test"))
    }
}

impl<'a, Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> AbstractNameService for TestContract<'a, Module, Error> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        Ok(AnsHost::new(Addr::unchecked("ans_host")))
    }
}

impl<'a, Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> DepsAccess for TestContract<'a, Module, Error> {
    fn deps_mut<'b: 'c, 'c>(&'b mut self) -> DepsMut<'c> {
        self.deps.branch()
    }
    fn deps<'b: 'c, 'c>(&'b self) -> Deps<'c> {
        self.deps.as_ref()
    }
}

impl<'a, Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> ExecutionStack for TestContract<'a, Module, Error> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.executable_stack
    }
}

impl<'a, Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> CustomEvents for TestContract<'a, Module, Error> {
    fn add_event(&mut self, event_name: &str, attributes: Vec<(&str, &str)>) {
        self
            .events
            .push(Event::new(event_name).add_attributes(attributes))
    }
}

// ANCHOR: interface_entry
// ANCHOR: entry_point_line
#[cfg_attr(feature = "export", entry_point)]
// ANCHOR_END: entry_point_line
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: Empty,
) -> AbstractSdkResult<Response> {
    let mut app = TestContract::new(deps.branch(), env, info);

    

    app.instantiate(None).unwrap();

    let resp = Response::new()
        .add_events(app.events.clone())
        .add_submessages(app._unwrap_for_response()?);
    Ok(resp)
}
