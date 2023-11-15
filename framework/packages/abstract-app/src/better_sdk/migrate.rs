use abstract_core::{app::{BaseInstantiateMsg, BaseMigrateMsg}, module_factory::{ContextResponse, QueryMsg as FactoryQuery, }, objects::module_version::{assert_contract_upgrade, set_module_data}};
use abstract_sdk::{namespaces::{ADMIN_NAMESPACE, BASE_STATE}, feature_objects::{VersionControlContract, AnsHost}, cw_helpers::wasm_smart_query, AbstractSdkResult};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Empty, CustomQuery, StdResult, StdError, Event, Addr};
use cw2::set_contract_version;
use cw_controllers::Admin;
use cw_storage_plus::Item;

use crate::state::AppState;

use super::{execution_stack::{DepsAccess, Executables, CustomEvents, ExecutionStack}, instantiate::AppBaseState, sdk::AccountIdentification, nameservice::AbstractNameService};


pub struct AppMigrateCtx<'a, C: CustomQuery = Empty>{
    pub deps: DepsMut<'a, C>,
    pub env: Env,

    pub base_state: AppBaseState,
    pub events: Vec<Event>,
    pub executables: Executables
}

impl<'a, C: CustomQuery> TryFrom<((DepsMut<'a, C>, Env), BaseMigrateMsg)> for AppMigrateCtx<'a, C> {

    type Error = StdError;

    fn try_from(((mut deps, env), base_msg): ((DepsMut<'a, C>, Env), BaseMigrateMsg)) -> StdResult<Self> {
        
        // let (name, version_string, metadata) = self.info();
        // let to_version = version_string.parse().unwrap();
        // assert_contract_upgrade(deps.storage, name, to_version)?;
        // set_module_data(
        //     deps.storage,
        //     name,
        //     version_string,
        //     self.dependencies(),
        //     metadata,
        // )?;
        // set_contract_version(deps.storage, name, version_string)?;

        let base_state = AppBaseState::default();
        // All the app logic
        Ok(Self { 
            deps, 
            env, 
            base_state,
            events: vec![],
            executables: Executables::default()
        })
    }

}

impl<'c> DepsAccess for AppMigrateCtx<'c, Empty>{
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b, Empty> {
        self.deps.branch()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> cosmwasm_std::Deps<'b, Empty> {
        self.deps.as_ref()
    }
}


impl<'a> CustomEvents for AppMigrateCtx<'a> {
    fn add_event(&mut self, event_name: &str, attributes: Vec<(&str, &str)>) {
        self
            .events
            .push(Event::new(event_name).add_attributes(attributes))
    }
    fn events(&self) -> Vec<Event>{
        self.events.clone()
    }
}

impl<'a> ExecutionStack for AppMigrateCtx<'a> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.executables
    }
}

impl<'a> AccountIdentification for AppMigrateCtx<'a> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(self.base_state.state.load(self.deps.storage)?.proxy_address)
    }
}


impl<'a> AbstractNameService for AppMigrateCtx<'a> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(self.base_state.state.load(self.deps.storage)?.ans_host)
    }
}
