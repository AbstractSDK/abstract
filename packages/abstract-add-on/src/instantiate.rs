use abstract_os::{
    add_on::{BaseInstantiateMsg, InstantiateMsg},
    module_factory::{ContextResponse, QueryMsg as FactoryQuery},
};
use cosmwasm_std::{
    to_binary, DepsMut, Env, MessageInfo, QueryRequest, Response, StdError, WasmQuery,
};

use abstract_sdk::{ans_host::AnsHost, Handler, InstantiateEndpoint};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    state::{AddOnContract, AddOnState},
    AddOnError,
};
use cw2::set_contract_version;

impl<
        Error: From<cosmwasm_std::StdError> + From<AddOnError>,
        CustomExecMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > InstantiateEndpoint
    for AddOnContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    type InstantiateMsg = InstantiateMsg<Self::CustomInitMsg>;
    fn instantiate(
        self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Error> {
        let BaseInstantiateMsg { ans_host_address } = msg.base;
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host_address)?,
        };

        // Caller is factory so get proxy and manager (admin) from there
        let resp: ContextResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: info.sender.to_string(),
            msg: to_binary(&FactoryQuery::Context {})?,
        }))?;

        let core = match resp.core {
            Some(core) => core,
            None => {
                return Err(
                    StdError::generic_err("context of module factory not properly set.").into(),
                )
            }
        };

        // Base state
        let state = AddOnState {
            proxy_address: core.proxy.clone(),
            ans_host,
        };
        let (name, version) = self.info();
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps.branch(), Some(core.manager))?;
        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new())
        };
        handler(deps, env, info, self, msg.app)
    }
}
