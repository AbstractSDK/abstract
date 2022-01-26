#![allow(unused_imports)]
#![allow(unused_variables)]

use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::commands;
use crate::error::AnchorError;
use crate::msg::{ExecuteMsg, QueryMsg};
use pandora::memory::item::Memory;
use pandora::treasury::dapp_base::commands::{self as dapp_base_commands, handle_base_init};
use pandora::treasury::dapp_base::common::BaseDAppResult;
use pandora::treasury::dapp_base::error::BaseDAppError;
use pandora::treasury::dapp_base::msg::BaseInstantiateMsg;
use pandora::treasury::dapp_base::queries as dapp_base_queries;
use pandora::treasury::dapp_base::state::BASESTATE;
use pandora::treasury::dapp_base::state::{BaseState, ADMIN};

pub type AnchorResult = Result<Response, BaseDAppError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: BaseInstantiateMsg,
) -> BaseDAppResult {
    let base_state = handle_base_init(deps.as_ref(), msg)?;

    BASESTATE.save(deps.storage, &base_state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> BaseDAppResult {
    match msg {
        ExecuteMsg::Base(message) => dapp_base_commands::handle_base_message(deps, info, message),
        // handle dapp-specific messages here
        // ExecuteMsg::Custom{} => commands::custom_command(),
        ExecuteMsg::DepositStable { deposit_amount } => {
            commands::handle_deposit_stable(deps.as_ref(), env, info, deposit_amount)
        }
        ExecuteMsg::RedeemStable { withdraw_amount } => {
            commands::handle_redeem_stable(deps.as_ref(), env, info, withdraw_amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => dapp_base_queries::handle_base_query(deps, message),
        // handle dapp-specific queries here
        // QueryMsg::Custom{} => queries::custom_query(),
    }
}
