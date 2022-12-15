use abstract_os::extension::ExtensionQueryMsg;
use abstract_sdk::base::{endpoints::QueryEndpoint, Handler};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};

use abstract_sdk::os::extension::{
    BaseQueryMsg, ExtensionConfigResponse, QueryMsg, TradersResponse,
};

use crate::{state::ExtensionContract, ExtensionError};

/// Where we dispatch the queries for the ExtensionContract
/// These ExtensionQueryMsg declarations can be found in `abstract_sdk::os::common_module::app_msg`
impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg: ExtensionQueryMsg,
        ReceiveMsg,
    > QueryEndpoint
    for ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
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
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
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

    fn dapp_config(&self, deps: Deps) -> StdResult<ExtensionConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(ExtensionConfigResponse {
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
