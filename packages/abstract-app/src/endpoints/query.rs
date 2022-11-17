use crate::{Handler, QueryEndpoint};
use abstract_sdk::os::app::{AppConfigResponse, BaseQueryMsg, QueryMsg};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};
use cw_controllers::AdminResponse;

use crate::{state::AppContract, AppError};

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > QueryEndpoint
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    type QueryMsg = QueryMsg<CustomQueryMsg>;

    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, StdError> {
        match msg {
            QueryMsg::Base(msg) => self.base_query(deps, env, msg),
            QueryMsg::App(msg) => self.query_handler()?(deps, env, self, msg),
        }
    }
}
/// Where we dispatch the queries for the AppContract
/// These BaseQueryMsg declarations can be found in `abstract_sdk::os::common_module::app_msg`
impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
    AppContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    pub fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::Admin {} => to_binary(&self.admin(deps)?),
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<AppConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        let admin = self.admin.get(deps)?.unwrap();
        Ok(AppConfigResponse {
            proxy_address: state.proxy_address,
            ans_host_address: state.ans_host.address,
            manager_address: admin,
        })
    }

    fn admin(&self, deps: Deps) -> StdResult<AdminResponse> {
        self.admin.query_admin(deps)
    }
}
