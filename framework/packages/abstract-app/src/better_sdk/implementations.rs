use abstract_sdk::{feature_objects::AnsHost, AbstractSdkResult};
use cosmwasm_std::{Addr, Deps, DepsMut, Empty, Env, Event, MessageInfo, Response};

use super::{
    execution_stack::{CustomEvents, DepsAccess, Executables, ExecutionStack},
    nameservice::AbstractNameService,
    sdk::AccountIdentification,
    TestContract,
};

// This is macro generated (because it has the struct in its types)
impl<'a> AccountIdentification for TestContract<'a> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(Addr::unchecked("proxy_test"))
    }
}

impl<'a> AbstractNameService for TestContract<'a> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        Ok(AnsHost::new(Addr::unchecked("ans_host")))
    }
}

impl<'a> DepsAccess for TestContract<'a> {
    fn deps_mut<'b: 'c, 'c>(&'b mut self) -> DepsMut<'c> {
        self.env.deps.branch()
    }
    fn deps<'b: 'c, 'c>(&'b self) -> Deps<'c> {
        self.env.deps.as_ref()
    }
}

impl<'a> ExecutionStack for TestContract<'a> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.env.executable_stack
    }
}

impl<'a> CustomEvents for TestContract<'a> {
    fn add_event(&mut self, event_name: &str, attributes: Vec<(&str, &str)>) {
        self.env
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
    let mut app: TestContract = TestContract::new(deps.branch(), env, info);

    app.instantiate(None).unwrap();

    let resp = Response::new()
        .add_events(app.env.events.clone())
        .add_submessages(app._unwrap_for_response()?);
    Ok(resp)
}
