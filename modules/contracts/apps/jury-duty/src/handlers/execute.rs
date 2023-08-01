#![allow(clippy::too_many_arguments)]

use cosmwasm_std::{CosmosMsg, DepsMut, Env, MessageInfo};
use cw3::Vote;
use cw3_flex_multisig::state::CONFIG;
use cw4::MemberChangedHookMsg;
use cw_utils::{Expiration, Threshold};

use abstract_sdk::features::AbstractResponse;
use abstract_sdk::NoisInterface;

use crate::contract::{AppResult, JuryDutyApp};
use crate::error::AppError;
use crate::msg::JuryDutyExecuteMsg;
use crate::state::JURIES;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: JuryDutyApp,
    msg: JuryDutyExecuteMsg,
) -> AppResult {
    Ok(match msg {
        JuryDutyExecuteMsg::Propose {
            title,
            description,
            msgs,
            latest,
        } => execute_random_propose(deps, env, info, app, title, description, msgs, latest)?,
        JuryDutyExecuteMsg::Vote { proposal_id, vote } => {
            execute_vote(deps, env, info, app, proposal_id, vote)?
        }
        JuryDutyExecuteMsg::Execute { proposal_id } => {
            cw3_flex_multisig::contract::execute_execute(deps, env, info, proposal_id)?
        }
        JuryDutyExecuteMsg::Close { proposal_id } => {
            cw3_flex_multisig::contract::execute_close(deps, env, info, proposal_id)?
        }
        JuryDutyExecuteMsg::MemberChangedHook(MemberChangedHookMsg { diffs }) => {
            cw3_flex_multisig::contract::execute_membership_hook(deps, env, info, diffs)?
        }
    })
}

fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _app: JuryDutyApp,
    proposal_id: u64,
    vote: Vote,
) -> AppResult {
    let jury = JURIES.may_load(deps.storage, &proposal_id)?;

    match jury {
        None => return Err(AppError::JuryNotSet(proposal_id)),
        Some(jury) => {
            let voter = info.sender.to_string();
            if !jury.contains(&voter) {
                return Err(AppError::NotJuryMember(voter));
            }
        }
    };

    Ok(cw3_flex_multisig::contract::execute_vote(
        deps,
        env,
        info,
        proposal_id,
        vote,
    )?)
}

const MAX_MEMBER_COUNT: u32 = 30;

pub fn execute_random_propose(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: JuryDutyApp,
    title: String,
    description: String,
    msgs: Vec<CosmosMsg>,
    latest: Option<Expiration>,
) -> AppResult {
    let cfg = CONFIG.load(deps.storage)?;

    // check that the threshold is absolute count
    match cfg.threshold {
        Threshold::AbsoluteCount { .. } => Ok(()),
        _ => Err(AppError::ThresholdMustBeAbsoluteCount),
    }?;

    let member_count = cfg
        .group_addr
        .list_members(&deps.querier, None, Some(MAX_MEMBER_COUNT))?
        .len();
    let total_weight = cfg.group_addr.total_weight(&deps.querier)?;

    // Check that the group is not too big
    if member_count >= MAX_MEMBER_COUNT as usize {
        return Err(AppError::TooManyMembers(MAX_MEMBER_COUNT));
    } else if total_weight >= member_count as u64 {
        return Err(AppError::MembersMustHaveWeightOfOne);
    }

    // Get the next proposal id
    let next_proposal_id = cw3_fixed_multisig::state::PROPOSAL_COUNT
        .may_load(deps.storage)?
        .unwrap_or_default();

    // Request randomness to get the callback
    let rand_request_msg = app
        .nois(deps.as_ref())?
        .next_randomness(next_proposal_id.to_string(), info.clone().funds)?;

    let propose_response = cw3_flex_multisig::contract::execute_propose(
        deps,
        env,
        info,
        title,
        description,
        msgs,
        latest,
    )?;
    Ok(app.tag_response(
        propose_response.add_messages(rand_request_msg),
        "random_propose",
    ))
}
