use crate::state::{AdapterContract, ContractError};
use abstract_core::{
    adapter::{
        AdapterConfigResponse, AdapterQueryMsg, AuthorizedAddressesResponse, BaseQueryMsg, QueryMsg,
    },
    objects::module_version::{ModuleDataResponse, MODULE},
};
use abstract_sdk::{
    base::{Handler, QueryEndpoint},
    features::DepsAccess,
};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult};

/// Where we dispatch the queries for the AdapterContract
/// These AdapterQueryMsg declarations can be found in `abstract_sdk::core::common_module::app_msg`
impl<
        'a,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg: AdapterQueryMsg,
        ReceiveMsg,
        SudoMsg,
    > QueryEndpoint
    for AdapterContract<
        'a,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    type QueryMsg = QueryMsg<CustomQueryMsg>;
    fn query(&self, msg: Self::QueryMsg) -> Result<Binary, Error> {
        match msg {
            QueryMsg::Module(msg) => self.query_handler()?(self, msg),
            QueryMsg::Base(msg) => self.base_query(msg),
        }
    }
}

impl<
        'a,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
        SudoMsg,
    >
    AdapterContract<'a, Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn base_query(&self, query: BaseQueryMsg) -> Result<Binary, Error> {
        match query {
            BaseQueryMsg::BaseConfig {} => {
                to_json_binary(&self.dapp_config().map_err(Error::from)?).map_err(Into::into)
            }
            BaseQueryMsg::AuthorizedAddresses { proxy_address } => {
                let proxy_address = self.api().addr_validate(&proxy_address)?;
                let authorized_addrs: Vec<Addr> = self
                    .authorized_addresses
                    .may_load(self.deps().storage, proxy_address)?
                    .unwrap_or_default();

                to_json_binary(&AuthorizedAddressesResponse {
                    addresses: authorized_addrs,
                })
                .map_err(Into::into)
            }
            BaseQueryMsg::ModuleData {} => {
                to_json_binary(&self.module_data().map_err(Error::from)?).map_err(Into::into)
            }
        }
    }

    fn dapp_config(&self) -> StdResult<AdapterConfigResponse> {
        let state = self.base_state.load(self.deps().storage)?;
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

    fn module_data(&self) -> StdResult<ModuleDataResponse> {
        let module_data = MODULE.load(self.deps().storage)?;
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
