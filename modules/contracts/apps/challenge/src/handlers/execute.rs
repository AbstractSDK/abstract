use crate::error::AppError;
use abstract_core::objects::voting::{ProposalInfo, ProposalOutcome, ProposalStatus, Vote};
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use abstract_sdk::{AbstractSdkResult, AccountVerification, Execution, TransferInterface};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::contract::{AppResult, ChallengeApp};
// use abstract_sdk::prelude::*;

use crate::msg::{ChallengeExecuteMsg, ChallengeRequest, Friend, VetoChallengeAction};
use crate::state::{
    ChallengeEntry, ChallengeEntryUpdate, UpdateFriendsOpKind, CHALLENGE_FRIENDS, CHALLENGE_LIST,
    NEXT_ID, SIMPLE_VOTING,
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    msg: ChallengeExecuteMsg,
) -> AppResult {
    match msg {
        ChallengeExecuteMsg::CreateChallenge { challenge_req } => {
            create_challenge(deps, env, info, app, challenge_req)
        }
        ChallengeExecuteMsg::UpdateChallenge {
            challenge_id,
            challenge,
        } => update_challenge(deps, env, info, app, challenge_id, challenge),
        ChallengeExecuteMsg::CancelChallenge { challenge_id } => {
            cancel_challenge(deps, env, info, &app, challenge_id)
        }
        ChallengeExecuteMsg::UpdateFriendsForChallenge {
            challenge_id,
            friends,
            op_kind,
        } => update_friends_for_challenge(deps, env, info, &app, challenge_id, friends, op_kind),
        ChallengeExecuteMsg::CastVote {
            vote_to_punish: vote,
            challenge_id,
        } => cast_vote(deps, env, info, &app, vote, challenge_id),
        ChallengeExecuteMsg::CountVotes { challenge_id } => {
            count_votes(deps, env, info, &app, challenge_id)
        }
        ChallengeExecuteMsg::VetoAction {
            challenge_id,
            action,
        } => veto_action(deps, env, info, &app, challenge_id, action),
    }
}

/// Create new challenge
fn create_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    challenge_req: ChallengeRequest,
) -> AppResult {
    // Only the admin should be able to create a challenge.
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    // Validate friend addr and account ids
    let friends_validated: Vec<(Addr, Friend<Addr>)> = challenge_req
        .init_friends
        .iter()
        .cloned()
        .map(|human| human.check(deps.as_ref(), &app))
        .collect::<AbstractSdkResult<_>>()?;

    let (initial_friends, friends): (Vec<Addr>, Vec<Friend<Addr>>) =
        friends_validated.into_iter().unzip();
    // Generate the challenge id
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &friends)?;

    // Create new proposal
    let end = challenge_req.duration.after(&env.block);
    let proposal_id = SIMPLE_VOTING.new_proposal(deps.storage, end, &initial_friends)?;

    // Create new challenge
    let challenge = ChallengeEntry::new(challenge_req, end, proposal_id);
    CHALLENGE_LIST.save(deps.storage, challenge_id, &challenge)?;

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "create_challenge",
    ))
}

fn update_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    challenge_id: u64,
    new_challenge: ChallengeEntryUpdate,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // will return an error if the challenge doesn't exist
    let mut loaded_challenge: ChallengeEntry = CHALLENGE_LIST
        .may_load(deps.storage, challenge_id)?
        .ok_or(AppError::ChallengeNotFound {})?;
    let proposal_id = loaded_challenge.current_proposal_id;

    SIMPLE_VOTING
        .load_proposal(deps.storage, proposal_id)?
        .assert_ready_for_action(&env.block)?;

    if let Some(name) = new_challenge.name {
        loaded_challenge.name = name;
    }

    if let Some(description) = new_challenge.description {
        loaded_challenge.description = description;
    }

    // Save the updated challenge
    CHALLENGE_LIST.save(deps.storage, challenge_id, &loaded_challenge)?;

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "update_challenge",
    ))
}

fn cancel_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let proposal_id = challenge.current_proposal_id;
    SIMPLE_VOTING.cancel_proposal(deps.storage, &env.block, proposal_id)?;

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "cancel_challenge",
    ))
}

fn update_friends_for_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    friends: Vec<Friend<String>>,
    op_kind: UpdateFriendsOpKind,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    // Validate friend addr and account ids
    let friends_validated: Vec<(Addr, Friend<Addr>)> = friends
        .iter()
        .cloned()
        .map(|human| human.check(deps.as_ref(), app))
        .collect::<AbstractSdkResult<_>>()?;

    let (voters_addrs, friends): (Vec<Addr>, Vec<Friend<Addr>>) =
        friends_validated.into_iter().unzip();

    let proposal_id = challenge.current_proposal_id;

    SIMPLE_VOTING
        .load_proposal(deps.storage, proposal_id)?
        .assert_ready_for_action(&env.block)?;

    match op_kind {
        UpdateFriendsOpKind::Add {} => {
            CHALLENGE_FRIENDS.update(deps.storage, challenge_id, |current_friends| {
                let mut current_friends = current_friends.ok_or(AppError::ZeroFriends {})?;
                current_friends.extend(friends.clone().into_iter());
                AppResult::Ok(current_friends)
            })?;
            SIMPLE_VOTING.add_voters(deps.storage, proposal_id, &env.block, &voters_addrs)?;
        }
        UpdateFriendsOpKind::Remove {} => {
            CHALLENGE_FRIENDS.update(deps.storage, challenge_id, |current_friends| {
                let mut current_friends = current_friends.ok_or(AppError::ZeroFriends {})?;
                for rem_friend in friends.iter() {
                    current_friends.retain(|friend| friend != rem_friend);
                }
                AppResult::Ok(current_friends)
            })?;
            SIMPLE_VOTING.remove_voters(deps.storage, proposal_id, &env.block, &voters_addrs)?;
        }
    }
    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "update_friends",
    ))
}

fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    vote: Vote,
    challenge_id: u64,
) -> AppResult {
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let proposal_id = challenge.current_proposal_id;
    let proposal_info =
        SIMPLE_VOTING.cast_vote(deps.storage, &env.block, proposal_id, &info.sender, vote)?;

    Ok(app
        .tag_response(Response::new(), "cast_vote")
        .add_attribute("proposal_info", format!("{proposal_info:?}")))
}

fn count_votes(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let proposal_id = challenge.current_proposal_id;
    let proposal_info = SIMPLE_VOTING.count_votes(deps.storage, &env.block, proposal_id)?;

    try_finish_vote(deps, env, app, proposal_info, challenge, challenge_id)
}

fn veto_action(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    action: VetoChallengeAction,
) -> AppResult {
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let proposal_id = challenge.current_proposal_id;

    let proposal_info = match action {
        VetoChallengeAction::AdminAction(action) => {
            app.admin.assert_admin(deps.as_ref(), &info.sender)?;
            SIMPLE_VOTING.veto_admin_action(deps.storage, &env.block, proposal_id, action)?
        }
        VetoChallengeAction::FinishExpired => {
            SIMPLE_VOTING
                .load_vote(deps.storage, proposal_id, &info.sender)?
                .ok_or(AppError::VoterNotFound {})?;
            SIMPLE_VOTING.finish_vote(deps.storage, &env.block, proposal_id)?
        }
    };

    try_finish_vote(deps, env, app, proposal_info, challenge, challenge_id)
}

fn try_finish_vote(
    deps: DepsMut,
    env: Env,
    app: &ChallengeApp,
    proposal_info: ProposalInfo,
    mut challenge: ChallengeEntry,
    challenge_id: u64,
) -> AppResult {
    let outcome = match &proposal_info.status {
        ProposalStatus::VetoPeriod(_, _) =>
        // veto period
        {
            return Ok(app
                .tag_response(Response::new(), "try_finish_vote")
                .add_attribute("proposal_info", format!("{proposal_info:?}")))
        }
        ProposalStatus::Finished(outcome) => outcome,
        ProposalStatus::Active => unreachable!(),
    };

    let friends = CHALLENGE_FRIENDS.load(deps.storage, challenge_id)?;
    let last_strike = if matches!(outcome, ProposalOutcome::Passed) {
        challenge.admin_strikes.strike()
    } else {
        false
    };

    // Create new voting if required
    if !last_strike && !challenge.end.is_expired(&env.block) {
        let initial_voters: Vec<Addr> = friends
            .iter()
            .map(|f| f.addr(deps.as_ref(), app))
            .collect::<AbstractSdkResult<_>>()?;
        let new_proposal_id =
            SIMPLE_VOTING.new_proposal(deps.storage, challenge.end, &initial_voters)?;
        challenge
            .previous_proposal_ids
            .push(challenge.current_proposal_id);
        challenge.current_proposal_id = new_proposal_id;
    };
    CHALLENGE_LIST.save(deps.storage, challenge_id, &challenge)?;

    // Return here if not required to charge penalty
    let res = if !matches!(outcome, ProposalOutcome::Passed) {
        app.tag_response(Response::new(), "finish_vote")
    } else {
        charge_penalty(deps, app, challenge, friends)?
    };
    Ok(res.add_attribute("proposal_info", format!("{proposal_info:?}")))
}

fn charge_penalty(
    deps: DepsMut,
    app: &ChallengeApp,
    challenge: ChallengeEntry,
    friends: Vec<Friend<Addr>>,
) -> Result<Response, AppError> {
    let num_friends = friends.len() as u128;
    if num_friends == 0 {
        return Err(AppError::ZeroFriends {});
    }
    let (amount_per_friend, remainder) = match challenge.strike_strategy {
        crate::state::StrikeStrategy::Split(amount) => (
            Uint128::new(amount.u128() / num_friends),
            amount.u128() % num_friends,
        ),
        crate::state::StrikeStrategy::PerFriend(amount) => (amount, 0),
    };

    let asset_per_friend = OfferAsset {
        name: challenge.strike_asset,
        amount: amount_per_friend,
    };

    let bank = app.bank(deps.as_ref());
    let executor = app.executor(deps.as_ref());

    // Create a transfer action for each friend
    let transfer_actions = friends
        .into_iter()
        .map(|friend| {
            let recipent = match friend {
                Friend::Addr(addr) => addr.address,
                Friend::AbstractAccount(account_id) => {
                    app.account_registry(deps.as_ref())
                        .account_base(&account_id)?
                        .proxy
                }
            };
            bank.transfer(vec![asset_per_friend.clone()], &recipent)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let transfer_msg = executor.execute(transfer_actions)?;

    Ok(app
        .tag_response(Response::new(), "charge_penalty")
        .add_message(transfer_msg)
        .add_attribute("remainder", remainder.to_string()))
}
