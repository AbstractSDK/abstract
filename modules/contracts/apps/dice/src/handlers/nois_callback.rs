use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use nois::{ints_in_range, NoisCallback};
use abstract_sdk::{AbstractResponse, NoisInterface};
use crate::contract::{AppResult, DiceApp};
use crate::state::DOUBLE_DICE_OUTCOME;

/// The execute_receive function is triggered upon reception of the randomness from the proxy contract
/// The callback contains the randomness from drand (HexBinary) and the job_id
pub fn nois_callback_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: DiceApp,
    callback: NoisCallback,
) -> AppResult {
    // In this Dapp we don't need the drand publish time. so we skip it with ..
    let NoisCallback {
        job_id, randomness, ..
    } = callback;

    let nois = app.nois(deps.as_ref())?;

    let randomness: [u8; 32] = nois.parse_randomness(randomness)?;

    //ints_in_range provides a list of random numbers following a uniform distribution within a range.
    //in this case it will provide uniformly randomized numbers between 1 and 6
    let double_dice_outcome = ints_in_range(randomness, 2, 1, 6);
    //summing the dice to fit the real double dice probability distribution from 2 to 12
    let double_dice_outcome = double_dice_outcome.iter().sum();

    // we've already checked that the job_id is not a duplicate
    DOUBLE_DICE_OUTCOME.save(deps.storage, &job_id, &double_dice_outcome)?;

    Ok(app.tag_response(Response::new(), "nois_callback"))
}