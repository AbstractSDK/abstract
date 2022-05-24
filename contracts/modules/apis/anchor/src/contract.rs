#![allow(unused_imports)]
#![allow(unused_variables)]

use abstract_api::{ApiContract, ApiError};
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use abstract_os::common_module::api_msg::ApiInstantiateMsg;
use abstract_os::native::memory::item::Memory;

use crate::commands;
use crate::msg::{ExecuteMsg, QueryMsg};

// no extra attrs
pub type AnchorApi<'a> = ApiContract<'a, Empty>;
pub type AnchorResult = Result<Response, ApiError>;

// use abstract_os::pandora_dapp::msg::ApiInstantiateMsg;
//
// use crate::commands;
// use crate::error::AnchorError;
// use crate::msg::{ExecuteMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInstantiateMsg,
) -> ApiResult {
    AnchorApi::default().instantiate(deps, env, info, msg, "anchor", "v1.1.0")?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ApiResult {
    let dapp = AnchorApi::default();

    match msg {
        ExecuteMsg::Base(message) => dapp.execute(deps, env, info, message),
        // handle dapp-specific messages here
        // ExecuteMsg::Custom{} => commands::custom_command(),
        ExecuteMsg::DepositStable { deposit_amount } => {
            commands::handle_deposit_stable(deps.as_ref(), env, info, dapp, deposit_amount)
        }
        ExecuteMsg::RedeemStable { withdraw_amount } => {
            commands::handle_redeem_stable(deps.as_ref(), env, info, dapp, withdraw_amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => AnchorApi::default().query(deps, env, message),
        // handle dapp-specific queries here
        // QueryMsg::Custom{} => queries::custom_query(),
    }
}
