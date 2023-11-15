use abstract_core::{app::BaseInstantiateMsg, module_factory::{ContextResponse, QueryMsg as FactoryQuery, }};
use abstract_sdk::{namespaces::{ADMIN_NAMESPACE, BASE_STATE}, feature_objects::{VersionControlContract, AnsHost}, cw_helpers::wasm_smart_query, AbstractSdkResult};
use abstract_testing::addresses::{TEST_MANAGER, TEST_PROXY};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Empty, CustomQuery, StdResult, StdError, Event, Addr};
use cw_controllers::Admin;
use cw_storage_plus::Item;

use crate::state::AppState;

use super::{execution_stack::{DepsAccess, CustomEvents, Executables, ExecutionStack}, sdk::AccountIdentification, nameservice::AbstractNameService};

pub struct AppBaseState{
    // Custom state for every App
    pub admin: Admin<'static>,
    pub(crate) state: Item<'static, AppState>,
}

impl Default for AppBaseState{
    fn default() -> Self{
        Self{
            state: Item::new(BASE_STATE),
            admin: Admin::new(ADMIN_NAMESPACE),
        }
    }
}

pub struct AppInstantiateCtx<'a, C: CustomQuery = Empty>{
    pub deps: DepsMut<'a, C>,
    pub env: Env,
    pub info: MessageInfo,

    pub base_state: AppBaseState,
    pub events: Vec<Event>,
    pub executables: Executables
}

impl<'a, C: CustomQuery> TryFrom<((DepsMut<'a, C>, Env, MessageInfo), BaseInstantiateMsg)> for AppInstantiateCtx<'a, C> {

    type Error = StdError;

    fn try_from(((mut deps, env, info), base_msg): ((DepsMut<'a, C>, Env, MessageInfo), BaseInstantiateMsg)) -> StdResult<Self> {
        
        let BaseInstantiateMsg {
            ans_host_address,
            version_control_address,
        } = base_msg;
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host_address)?,
        };
        let version_control = VersionControlContract {
            address: deps.api.addr_validate(&version_control_address)?,
        };

        // TODO: Would be nice to remove context
        // Issue: We can't pass easily AccountBase with BaseInstantiateMsg(right now)

        // Caller is factory so get proxy and manager (admin) from there
        // let resp: ContextResponse = deps.querier.query(&wasm_smart_query(
        //     info.sender.to_string(),
        //     &FactoryQuery::Context {},
        // )?)?;
         let resp = ContextResponse{
            account_base:abstract_core::version_control::AccountBase { 
                manager: Addr::unchecked(TEST_MANAGER),
                proxy: Addr::unchecked(TEST_PROXY) 
            }
        };

        let account_base = resp.account_base;

        // Base state
        let state = AppState {
            proxy_address: account_base.proxy.clone(),
            ans_host,
            version_control,
        };

        // let (name, version, metadata) = self.info();
        // set_module_data(deps.storage, name, version, self.dependencies(), metadata)?;
        // set_contract_version(deps.storage, name, version)?;

        let base_state = AppBaseState::default();
        base_state.state.save(deps.storage, &state)?;
        base_state.admin.set(deps.branch(), Some(account_base.manager))?;

        // All the app logic
        Ok(Self { 
            deps, 
            env, 
            info,
            base_state,
            events: vec![],
            executables: Executables::default()
        })
    }

}

impl<'c> DepsAccess for AppInstantiateCtx<'c, Empty>{
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b, Empty> {
        self.deps.branch()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> cosmwasm_std::Deps<'b, Empty> {
        self.deps.as_ref()
    }
}

impl<'a> CustomEvents for AppInstantiateCtx<'a> {
    fn add_event(&mut self, event_name: &str, attributes: Vec<(&str, &str)>) {
        self
            .events
            .push(Event::new(event_name).add_attributes(attributes))
    }
    fn events(&self) -> Vec<Event>{
        self.events.clone()
    }
}

impl<'a> ExecutionStack for AppInstantiateCtx<'a> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.executables
    }
}

impl<'a> AccountIdentification for AppInstantiateCtx<'a> {
    fn proxy_address(&self) -> AbstractSdkResult<Addr> {
        Ok(self.base_state.state.load(self.deps.storage)?.proxy_address)
    }
}


impl<'a> AbstractNameService for AppInstantiateCtx<'a> {
    fn ans_host(&self) -> AbstractSdkResult<AnsHost> {
        // Retrieve the ANS host address from the base state.
        Ok(self.base_state.state.load(self.deps.storage)?.ans_host)
    }
}
