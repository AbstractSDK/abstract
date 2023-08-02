#![allow(clippy::too_many_arguments)]

use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response,
};

use abstract_sdk::NoisInterface;
use abstract_sdk::features::AbstractResponse;

use crate::contract::{AppResult, DiceApp};
use crate::msg::{DiceExecuteMsg};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DiceApp,
    msg: DiceExecuteMsg,
) -> AppResult {
    match msg {
        //RollDice should be called by a player who wants to roll the dice
        DiceExecuteMsg::RollDice { job_id } => execute_roll_dice(deps, env, info, app, job_id),
    }
}

//execute_roll_dice is the function that will trigger the process of requesting randomness.
//The request from randomness happens by calling the nois-proxy contract
pub fn execute_roll_dice(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: DiceApp,
    job_id: String,
) -> AppResult {

    let nois = app.nois(deps.as_ref())?;

    // TODO: pay funds from the account
    let rand_msg = nois.next_randomness(job_id, info.funds)?;
    Ok(app.tag_response(Response::new().add_messages(rand_msg) , "roll_dice"))
}