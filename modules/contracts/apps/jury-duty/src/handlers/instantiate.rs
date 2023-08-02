use cosmwasm_std::{DepsMut, Env, MessageInfo};

use crate::contract::{AppResult, JuryDutyApp};
use crate::error::AppError;
use crate::msg::JuryDutyInstantiateMsg;

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _app: JuryDutyApp,
    msg: JuryDutyInstantiateMsg,
) -> AppResult {
    // Validate that each jury member has the same weight
    let weight = msg.voters[0].weight;
    for voter in &msg.voters {
        if voter.weight != weight {
            return Err(AppError::MembersMustHaveSameWeight);
        }
    }
    Ok(cw3_fixed_multisig::contract::instantiate(
        deps, env, info, msg,
    )?)
}
