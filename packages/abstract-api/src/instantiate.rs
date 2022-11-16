use abstract_os::api::InstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use abstract_sdk::{ans_host::AnsHost, Handler, InstantiateEndpoint};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    state::{ApiContract, ApiState},
    ApiError,
};

use cw2::set_contract_version;

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg,
        ReceiveMsg,
    > InstantiateEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    type InstantiateMsg = InstantiateMsg<CustomInitMsg>;
    /// Instantiate the API
    fn instantiate(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Error> {
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&msg.base.ans_host_address)?,
        };

        // Base state
        let state = ApiState {
            version_control: deps.api.addr_validate(&msg.base.version_control_address)?,
            ans_host,
        };
        let (name, version) = self.info();
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new())
        };
        handler(deps, env, info, self, msg.app)
    }
}
