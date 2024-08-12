use abstract_sdk::base::{Handler, QueryEndpoint};
use abstract_std::{
    adapter::{
        AdapterConfigResponse, AdapterQueryMsg, AuthorizedAddressesResponse, BaseQueryMsg, QueryMsg,
    },
    objects::module_version::{ModuleDataResponse, MODULE},
};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult};

use crate::state::{AdapterContract, ContractError};

/// Where we dispatch the queries for the AdapterContract
/// These AdapterQueryMsg declarations can be found in `abstract_sdk::std::common_module::app_msg`
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg: AdapterQueryMsg,
        SudoMsg,
    > QueryEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    type QueryMsg = QueryMsg<CustomQueryMsg>;
    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, Error> {
        match msg {
            QueryMsg::Module(msg) => self.query_handler()?(deps, env, self, msg),
            QueryMsg::Base(msg) => self.base_query(deps, env, msg),
        }
    }
}

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> Result<Binary, Error> {
        match query {
            BaseQueryMsg::BaseConfig {} => {
                to_json_binary(&self.dapp_config(deps).map_err(Error::from)?).map_err(Into::into)
            }
            BaseQueryMsg::AuthorizedAddresses { proxy_address } => {
                let proxy_address = deps.api.addr_validate(&proxy_address)?;
                let authorized_addrs: Vec<Addr> = self
                    .authorized_addresses
                    .may_load(deps.storage, proxy_address)?
                    .unwrap_or_default();

                to_json_binary(&AuthorizedAddressesResponse {
                    addresses: authorized_addrs,
                })
                .map_err(Into::into)
            }
            BaseQueryMsg::ModuleData {} => {
                to_json_binary(&self.module_data(deps).map_err(Error::from)?).map_err(Into::into)
            }
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<AdapterConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(AdapterConfigResponse {
            version_control_address: state.version_control.address,
            ans_host_address: state.ans_host.address,
            dependencies: self
                .dependencies()
                .iter()
                .map(|dep| dep.id.to_string())
                .collect(),
        })
    }

    fn module_data(&self, deps: Deps) -> StdResult<ModuleDataResponse> {
        let module_data = MODULE.load(deps.storage)?;
        Ok(ModuleDataResponse {
            module_id: module_data.module,
            version: module_data.version,
            dependencies: module_data
                .dependencies
                .into_iter()
                .map(Into::into)
                .collect(),
            metadata: module_data.metadata,
        })
    }
}
