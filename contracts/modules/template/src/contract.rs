#![allow(unused_imports)]
#![allow(unused_variables)]

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use pandora_dapp_base::{DappContract, DappError, DappResult};
use pandora_os::pandora_dapp::msg::DappInstantiateMsg;

use crate::commands;
use crate::msg::{ExecuteMsg, QueryMsg};

type TemplateExtension = Option<Empty>;
pub type TemplateDapp<'a> = DappContract<'a, TemplateExtension, Empty>;
// Should include TemplateError instead of DappError
pub type TemplateResult = Result<Response, DappError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: DappInstantiateMsg,
) -> DappResult {
    TemplateDapp::default().instantiate(deps, env, info, msg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> TemplateResult {
    let dapp = TemplateDapp::default();
    match msg {
        // handle dapp-specific messages here
        // ExecuteMsg::Custom{} => commands::custom_command(),
        ExecuteMsg::Base(dapp_msg) => {
            from_base_dapp_result(dapp.execute(deps, env, info, dapp_msg))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => TemplateDapp::default().query(deps, env, message),
        // handle dapp-specific queries here
        // QueryMsg::Custom{} => queries::custom_query(),
    }
}

/// Required to convert BaseDAppResult into TerraswapResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: DappResult) -> TemplateResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
