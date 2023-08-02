use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Order, Response, StdError};
use cw3_fixed_multisig::state::CONFIG;
use cw_utils::Threshold;
use nois::{pick, NoisCallback};

use abstract_sdk::{AbstractResponse, NoisInterface};

use crate::contract::{AppResult, JuryDutyApp};
use crate::error::AppError;
use crate::state::JURIES;

/// The execute_receive function is triggered upon reception of the randomness from the proxy contract
/// The callback contains the randomness from drand (HexBinary) and the job_id
pub fn nois_callback_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: JuryDutyApp,
    callback: NoisCallback,
) -> AppResult {
    let NoisCallback {
        job_id, randomness, ..
    } = callback;

    let nois = app.nois(deps.as_ref())?;
    let randomness: [u8; 32] = nois.parse_randomness(randomness)?;

    let proposal_id = job_id.parse::<u64>().unwrap();
    let cfg = CONFIG.load(deps.storage)?;

    // check that the threshold is absolute count
    let threshold = match cfg.threshold {
        Threshold::AbsoluteCount { weight } => Ok(weight),
        _ => Err(AppError::ThresholdMustBeAbsoluteCount),
    }?;

    let member_addrs = cw3_fixed_multisig::state::VOTERS.keys(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<Addr>, StdError>>()?;

    let jury = pick(randomness, threshold as usize, member_addrs);

    JURIES.save(deps.storage, &proposal_id, &jury)?;

    Ok(app.tag_response(Response::new(), "nois_callback"))
}
