use abstract_sdk::os::extension::InstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use abstract_sdk::{
    base::{endpoints::InstantiateEndpoint, Handler},
    feature_objects::AnsHost,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    state::{ExtensionContract, ExtensionState},
    ExtensionError,
};

use cw2::set_contract_version;

impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg,
        ReceiveMsg,
    > InstantiateEndpoint
    for ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    type InstantiateMsg = InstantiateMsg<CustomInitMsg>;
    /// Instantiate the extension
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
        let state = ExtensionState {
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
