use abstract_app::sdk::base::CustomExecuteHandler;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{error::EtfError, msg::CustomExecuteMsg};

impl CustomExecuteHandler<crate::contract::EtfApp> for CustomExecuteMsg {
    type ExecuteMsg = crate::msg::ExecuteMsg;

    fn try_into_base(self) -> Result<Self::ExecuteMsg, Self> {
        match self {
            CustomExecuteMsg::Base(msg) => Ok(crate::msg::ExecuteMsg::from(msg)),
            CustomExecuteMsg::Module(msg) => Ok(crate::msg::ExecuteMsg::from(msg)),
            _ => Err(self),
        }
    }

    fn custom_execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        module: crate::contract::EtfApp,
    ) -> Result<Response, EtfError> {
        match self {
            CustomExecuteMsg::Receive(cw20_msg) => {
                super::receive_cw20(deps, env, info, module, cw20_msg)
            }
            _ => unreachable!(),
        }
    }
}
