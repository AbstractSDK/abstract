use std::collections::HashSet;

use crate::error::AppError;
use abstract_core::objects::voting::{ProposalId, ProposalInfo, ProposalOutcome, Vote};
use abstract_core::objects::AnsAsset;
use abstract_sdk::features::AbstractResponse;
use abstract_sdk::{AbstractSdkResult, AccountVerification, Execution, TransferInterface};
use cosmwasm_std::{
    ensure, Addr, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdResult, Uint128,
};

use crate::contract::{AppResult, ChallengeApp};

use crate::msg::{ChallengeExecuteMsg, ChallengeRequest, Friend};
use crate::state::{
    ChallengeEntry, ChallengeEntryUpdate, UpdateFriendsOpKind, CHALLENGES, CHALLENGE_FRIENDS,
    CHALLENGE_PROPOSALS, MAX_AMOUNT_OF_FRIENDS, NEXT_ID, SIMPLE_VOTING,
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
        ChallengeExecuteMsg::Veto { challenge_id } => veto(deps, env, info, &app, challenge_id),
        ChallengeExecuteMsg::UpdateConfig { new_vote_config } => {
            SIMPLE_VOTING.update_vote_config(deps.storage, &new_vote_config)?;
            Ok(Response::new())
        }
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
    ensure!(
        challenge_req.init_friends.len() < MAX_AMOUNT_OF_FRIENDS as usize,
        AppError::TooManyFriends {}
    );
    // Validate friend addr and account ids
    let friends_validated: Vec<(Addr, Friend<Addr>)> = challenge_req
        .init_friends
        .iter()
        .cloned()
        .map(|human| human.check(deps.as_ref(), &app))
        .collect::<AbstractSdkResult<_>>()?;

    let (friend_addrs, friends): (Vec<Addr>, Vec<Friend<Addr>>) =
        friends_validated.into_iter().unzip();
    // Check if addrs unique
    let mut unique_addrs = HashSet::with_capacity(friend_addrs.len());
    if !friend_addrs.iter().all(|x| unique_addrs.insert(x)) {
        return Err(AppError::DuplicateFriends {});
    }

    // Generate the challenge id
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &friends)?;

    // Create new challenge
    let end_timestamp = env
        .block
        .time
        .plus_seconds(challenge_req.challenge_duration_seconds.u64());
    let challenge = ChallengeEntry::new(challenge_req, end_timestamp)?;
    CHALLENGES.save(deps.storage, challenge_id, &challenge)?;

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "create_challenge",
    ))
}

fn update_challenge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    challenge_id: u64,
    new_challenge: ChallengeEntryUpdate,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // will return an error if the challenge doesn't exist
    let mut loaded_challenge: ChallengeEntry = CHALLENGES
        .may_load(deps.storage, challenge_id)?
        .ok_or(AppError::ChallengeNotFound {})?;

    // TODO: are we ok to edit name/description during proposals?
    if let Some(name) = new_challenge.name {
        loaded_challenge.name = name;
    }

    if let Some(description) = new_challenge.description {
        loaded_challenge.description = description;
    }

    // Save the updated challenge
    CHALLENGES.save(deps.storage, challenge_id, &loaded_challenge)?;

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
    let mut challenge = CHALLENGES.load(deps.storage, challenge_id)?;
    // Check if this challenge still active
    if env.block.time >= challenge.end_timestamp {
        return Err(AppError::ChallengeExpired {});
    }

    // If there is active proposal - cancel it
    let last_proposal_id = last_proposal(challenge_id, deps.as_ref())?;
    if let Some(proposal_id) = last_proposal_id {
        SIMPLE_VOTING.cancel_proposal(deps.storage, &env.block, proposal_id)?;
    }

    // End it now
    challenge.end_timestamp = env.block.time;
    CHALLENGES.save(deps.storage, challenge_id, &challenge)?;

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
    // Validate friend addr and account ids
    let friends_validated: Vec<(Addr, Friend<Addr>)> = friends
        .iter()
        .cloned()
        .map(|human| human.check(deps.as_ref(), app))
        .collect::<AbstractSdkResult<_>>()?;

    let (voters_addrs, friends): (Vec<Addr>, Vec<Friend<Addr>>) =
        friends_validated.into_iter().unzip();

    let last_proposal_id = last_proposal(challenge_id, deps.as_ref())?;

    // Don't allow edit friends if last proposal haven't ended yet
    if let Some(proposal_id) = last_proposal_id {
        let info = SIMPLE_VOTING.load_proposal(deps.storage, &env.block, proposal_id)?;
        if env.block.time < info.end_timestamp {
            return Err(AppError::FriendsEditDuringProposal(info.end_timestamp));
        }
    }

    match op_kind {
        UpdateFriendsOpKind::Add {} => {
            let mut current_friends = CHALLENGE_FRIENDS
                .may_load(deps.storage, challenge_id)?
                .ok_or(AppError::ChallengeNotFound {})?;

            ensure!(
                friends.len() + current_friends.len() < MAX_AMOUNT_OF_FRIENDS as usize,
                AppError::TooManyFriends {}
            );

            let mut current_friends_addrs: Vec<Addr> = current_friends
                .iter()
                .map(|f| f.addr(deps.as_ref(), app))
                .collect::<AbstractSdkResult<_>>()?;
            current_friends_addrs.extend(voters_addrs);
            // Check if addrs unique
            let mut unique_addrs = HashSet::with_capacity(current_friends_addrs.len());
            if !current_friends_addrs.iter().all(|x| unique_addrs.insert(x)) {
                return Err(AppError::DuplicateFriends {});
            }

            current_friends.extend(friends);
            CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &current_friends)?;
        }
        UpdateFriendsOpKind::Remove {} => {
            CHALLENGE_FRIENDS.update(deps.storage, challenge_id, |current_friends| {
                let mut current_friends = current_friends.ok_or(AppError::ZeroFriends {})?;
                for rem_friend in friends.iter() {
                    current_friends.retain(|friend| friend != rem_friend);
                }
                AppResult::Ok(current_friends)
            })?;
        }
    }
    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "update_friends",
    ))
}

fn get_or_create_active_proposal(
    deps: &mut DepsMut,
    env: &Env,
    challenge_id: u64,
    app: &ChallengeApp,
) -> AppResult<ProposalId> {
    let challenge = CHALLENGES.load(deps.storage, challenge_id)?;

    // Load last proposal and use it if it's active
    if let Some(proposal_id) = last_proposal(challenge_id, deps.as_ref())? {
        let proposal = SIMPLE_VOTING.load_proposal(deps.storage, &env.block, proposal_id)?;
        if proposal.assert_active_proposal().is_ok() {
            return Ok(proposal_id);
        }
    }

    // Or create a new one otherwise
    if env.block.time >= challenge.end_timestamp {
        return Err(AppError::ChallengeExpired {});
    }
    let friends: Vec<Addr> = CHALLENGE_FRIENDS
        .load(deps.storage, challenge_id)?
        .into_iter()
        .map(|friend| friend.addr(deps.as_ref(), app))
        .collect::<AbstractSdkResult<_>>()?;
    let proposal_id = SIMPLE_VOTING.new_proposal(
        deps.storage,
        env.block
            .time
            .plus_seconds(challenge.proposal_duration_seconds.u64()),
        &friends,
    )?;
    CHALLENGE_PROPOSALS.save(deps.storage, (challenge_id, proposal_id), &Empty {})?;

    Ok(proposal_id)
}

fn cast_vote(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    vote: Vote,
    challenge_id: u64,
) -> AppResult {
    let proposal_id = get_or_create_active_proposal(&mut deps, &env, challenge_id, app)?;

    let voter = match app
        .account_registry(deps.as_ref())
        .assert_proxy(&info.sender)
    {
        Ok(base) => base.manager,
        Err(_) => info.sender,
    };
    let proposal_info =
        SIMPLE_VOTING.cast_vote(deps.storage, &env.block, proposal_id, &voter, vote)?;

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
    let challenge = CHALLENGES.load(deps.storage, challenge_id)?;
    let proposal_id =
        last_proposal(challenge_id, deps.as_ref())?.ok_or(AppError::ExpectedProposal {})?;
    let (proposal_info, outcome) =
        SIMPLE_VOTING.count_votes(deps.storage, &env.block, proposal_id)?;

    try_finish_challenge(
        deps,
        env,
        app,
        proposal_info,
        outcome,
        challenge,
        challenge_id,
    )
}

fn veto(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    let proposal_id =
        last_proposal(challenge_id, deps.as_ref())?.ok_or(AppError::ExpectedProposal {})?;

    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let proposal_info = SIMPLE_VOTING.veto_proposal(deps.storage, &env.block, proposal_id)?;

    Ok(app.tag_response(
        Response::new().add_attribute("proposal_info", format!("{proposal_info:?}")),
        "veto",
    ))
}

fn try_finish_challenge(
    deps: DepsMut,
    _env: Env,
    app: &ChallengeApp,
    proposal_info: ProposalInfo,
    proposal_outcome: ProposalOutcome,
    mut challenge: ChallengeEntry,
    challenge_id: u64,
) -> AppResult {
    let friends = CHALLENGE_FRIENDS.load(deps.storage, challenge_id)?;
    let challenge_finished = if matches!(proposal_outcome, ProposalOutcome::Passed) {
        challenge.admin_strikes.strike()
    } else {
        false
    };

    // Return here if not required to charge penalty
    let res = if !matches!(proposal_outcome, ProposalOutcome::Passed) {
        app.tag_response(Response::new(), "finish_vote")
    } else {
        charge_penalty(deps, app, challenge, friends)?
    };
    Ok(res
        .add_attribute("proposal_info", format!("{proposal_info:?}"))
        .add_attribute("challenge_finished", challenge_finished.to_string()))
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

    let asset_per_friend = AnsAsset {
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

pub(crate) fn last_proposal(challenge_id: u64, deps: Deps) -> StdResult<Option<ProposalId>> {
    CHALLENGE_PROPOSALS
        .prefix(challenge_id)
        .keys(deps.storage, None, None, Order::Descending)
        .next()
        .transpose()
}
