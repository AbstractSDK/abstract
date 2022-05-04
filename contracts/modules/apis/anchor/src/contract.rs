#![allow(unused_imports)]
#![allow(unused_variables)]

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use pandora_dapp_base::{DappContract, DappError, DappResult};
use pandora_os::native::memory::item::Memory;
use pandora_os::pandora_dapp::msg::DappInstantiateMsg;

use crate::commands;
use crate::msg::{ExecuteMsg, QueryMsg};

// no extra attrs
type AnchorExtension = Option<Empty>;
pub type AnchorDapp<'a> = DappContract<'a, AnchorExtension, Empty>;
pub type AnchorResult = Result<Response, DappError>;

// use pandora_os::pandora_dapp::msg::DappInstantiateMsg;
//
// use crate::commands;
// use crate::error::AnchorError;
// use crate::msg::{ExecuteMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: DappInstantiateMsg,
) -> DappResult {
    AnchorDapp::default().instantiate(deps, env, info, msg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DappResult {
    let dapp = AnchorDapp::default();

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
        QueryMsg::Base(message) => AnchorDapp::default().query(deps, env, message),
        // handle dapp-specific queries here
        // QueryMsg::Custom{} => queries::custom_query(),
    }
}
