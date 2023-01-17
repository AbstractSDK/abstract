use crate::{state::ApiContract, ApiError};
use abstract_os::api::ApiQueryMsg;
use abstract_sdk::base::{endpoints::QueryEndpoint, Handler};
use abstract_sdk::os::api::{ApiConfigResponse, BaseQueryMsg, QueryMsg, TradersResponse};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};

/// Where we dispatch the queries for the ApiContract
/// These ApiQueryMsg declarations can be found in `abstract_sdk::os::common_module::app_msg`
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg: ApiQueryMsg,
        ReceiveMsg,
    > QueryEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    type QueryMsg = QueryMsg<CustomQueryMsg>;
    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, StdError> {
        match msg {
            QueryMsg::App(msg) => self.query_handler()?(deps, env, self, msg),
            QueryMsg::Base(msg) => self.base_query(deps, env, msg),
        }
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::Traders { proxy_address } => {
                let traders = self
                    .traders
                    .may_load(deps.storage, deps.api.addr_validate(&proxy_address)?)?
                    .unwrap_or_default();
                to_binary(&TradersResponse {
                    traders: traders.into_iter().collect(),
                })
            }
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<ApiConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(ApiConfigResponse {
            version_control_address: state.version_control,
            ans_host_address: state.ans_host.address,
            dependencies: self
                .dependencies()
                .iter()
                .map(|dep| dep.id.to_string())
                .collect(),
        })
    }
}
