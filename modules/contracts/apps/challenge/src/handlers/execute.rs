use crate::error::AppError;
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::contract::{AppResult, ChallengeApp};
use abstract_sdk::prelude::*;

use crate::msg::ChallengeExecuteMsg;
use crate::state::{
    ChallengeEntry, ChallengeEntryUpdate, CheckIn, Friend, Penalty, UpdateFriendsOpKind, Vote,
    ADMIN, CHALLENGE_FRIENDS, CHALLENGE_LIST, DAILY_CHECK_INS, NEXT_ID, VOTES,
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    msg: ChallengeExecuteMsg,
) -> AppResult {
    match msg {
        ChallengeExecuteMsg::CreateChallenge { challenge } => {
            create_challenge(deps, env, info, app, challenge)
        }
        ChallengeExecuteMsg::UpdateChallenge {
            challenge_id,
            challenge,
        } => update_challenge(deps, env, info, app, challenge_id, challenge),
        ChallengeExecuteMsg::CancelChallenge { challenge_id } => {
            cancel_challenge(deps, info, app, challenge_id)
        }
        ChallengeExecuteMsg::UpdateFriendsForChallenge {
            challenge_id,
            friends,
            op_kind,
        } => update_friends_for_challenge(deps, info, &app, challenge_id, friends, op_kind),
        ChallengeExecuteMsg::DailyCheckIn {
            challenge_id,
            metadata,
        } => daily_check_in(deps, env, info, &app, challenge_id, metadata),
        ChallengeExecuteMsg::CastVote { vote, challenge_id } => {
            cast_vote(deps, env, info, &app, vote, challenge_id)
        }
        ChallengeExecuteMsg::CountVotes { challenge_id } => {
            count_votes(deps, info, env, &app, challenge_id)
        }
        ChallengeExecuteMsg::VetoVote { vote, challenge_id } => {
            veto_vote(deps, info, env, challenge_id, vote)
        }
    }
}

/// Create new challenge
fn create_challenge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    challenge: ChallengeEntry,
) -> AppResult {
    // Only the admin should be able to create a challenge.
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Generate the challenge id
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

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
    let mut loaded_challenge: ChallengeEntry = CHALLENGE_LIST
        .may_load(deps.storage, challenge_id.clone())
        .map_err(|_| {
            AppError::Std(StdError::generic_err(format!(
                "Error loading challenge with id {}",
                challenge_id
            )))
        })?
        .ok_or_else(|| {
            AppError::Std(StdError::generic_err(format!(
                "Challenge with id {} not found",
                challenge_id
            )))
        })?;

    if let Some(name) = new_challenge.name {
        loaded_challenge.name = name;
    }
    if let Some(collateral) = new_challenge.collateral {
        loaded_challenge.collateral = collateral;
    }
    if let Some(description) = new_challenge.description {
        loaded_challenge.description = description;
    }

    // Save the updated challenge
    CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &loaded_challenge)?;

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "update_challenge",
    ))
}

fn cancel_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    CHALLENGE_LIST.remove(deps.storage, challenge_id.clone());

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "cancel_challenge",
    ))
}

fn update_friends_for_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    friends: Vec<Friend<String>>,
    op_kind: UpdateFriendsOpKind,
) -> AppResult {
    match op_kind {
        UpdateFriendsOpKind::Add => {
            add_friends_for_challenge(deps, info, app, challenge_id, friends)
        }
        UpdateFriendsOpKind::Remove => {
            remove_friends_from_challenge(deps, info, app, challenge_id, friends)
        }
    }
}

fn add_friends_for_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    friends: Vec<Friend<String>>,
) -> AppResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut existing_friends = CHALLENGE_FRIENDS
        .may_load(deps.storage, challenge_id.clone())?
        .unwrap_or_default();

    for friend in &friends {
        if existing_friends.iter().any(|f| f.address == friend.address) {
            return Err(AppError::AlreadyAdded {});
        }
    }

    // validate the String addresses and convert them to Addr
    // before saving
    let friends: Vec<Friend<Addr>> = friends
        .iter()
        .cloned()
        .map(|friend| friend.check(deps.as_ref()).unwrap())
        .collect();

    existing_friends.extend(friends);
    CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &existing_friends)?;
    Ok(Response::new().add_attribute("action", "add_friends"))
}

pub fn remove_friends_from_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    friend_addresses: Vec<Friend<String>>,
) -> AppResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut existing_friends = CHALLENGE_FRIENDS
        .may_load(deps.storage, challenge_id.clone())?
        .unwrap_or_default();

    for friend in &friend_addresses {
        if !existing_friends.iter().any(|f| f.address == friend.address) {
            return Err(AppError::Std(StdError::generic_err(
                "Friend not found for this challenge",
            )));
        }
    }

    existing_friends.retain(|f| {
        !friend_addresses
            .iter()
            .any(|friend| f.address == friend.address)
    });
    CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &existing_friends)?;
    Ok(Response::new().add_attribute("action", "remove_friends"))
}

fn daily_check_in(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    metadata: Option<String>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let now = env.block.time.seconds();
    // Check if Admin has already checked in today
    if DAILY_CHECK_INS
        .load(deps.storage, now.clone())
        .map_or(false, |check_in| check_in.last_checked_in == now.clone())
    {
        return Err(AppError::AlreadyCheckedIn {});
    }

    // this could be configurable, for now
    // we set the next check in to be 100 blocks from now
    let next_check_in_block = env.block.height + 100;
    let check_in = CheckIn {
        last_checked_in: now,
        next_check_in_by: next_check_in_block,
        metadata,
    };

    DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_in)?;
    Ok(Response::new().add_attribute("action", "check_in"))
}

fn cast_vote(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _app: &ChallengeApp,
    vote: Vote<String>,
    challenge_id: u64,
) -> AppResult {
    let vote = vote.check(deps.as_ref())?;

    let now = env.block.time.seconds();

    // If Admin checked in today, friends can't vote
    if DAILY_CHECK_INS
        .load(deps.storage, now.clone())
        .map_or(false, |check_in| check_in.last_checked_in == now)
    {
        return Err(AppError::AlreadyCheckedIn {});
    }

    let vote = vote.optimisitc();

    // check that the vote.voter has note has not voted
    if VOTES
        .may_load(
            deps.storage,
            (challenge_id.to_owned(), vote.voter.to_owned()),
        )
        .map_or(false, |votes| votes.iter().any(|v| v.voter == vote.voter))
    {
        return Err(AppError::AlreadyVoted {});
    } else {
        VOTES.save(
            deps.storage,
            (challenge_id.to_owned(), vote.voter.to_owned()),
            &vote,
        )?;
    }
    Ok(Response::new().add_attribute("action", "cast_vote"))
}

fn count_votes(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    let votes = VOTES.may_load(
        deps.storage,
        (challenge_id.to_owned(), info.sender.to_owned()),
    )?;

    let any_false_vote = votes.iter().any(|v| v.approval == Some(false));
    if any_false_vote {
        return charge_penalty(deps, info, app, challenge_id);
    }

    Ok(Response::new().add_attribute("action", "count_votes"))
}

fn veto_vote(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    challenge_id: u64,
    vote: Vote<String>,
) -> AppResult {
    if info.sender.to_string() != ADMIN.load(deps.storage)? {
        return Err(AppError::Std(StdError::generic_err(
            "Only the admin can veto a vote",
        )));
    }

    let vote = vote.check(deps.as_ref())?;
    let mut fetched_vote = VOTES.may_load(
        deps.storage,
        (challenge_id.to_owned(), vote.voter.to_owned()),
    )?;

    // we set the vote the opposite to what it currently is
    if let Some(v) = &mut fetched_vote {
        v.approval = Some(!v.approval.unwrap());
        VOTES.remove(deps.storage, (challenge_id.clone(), vote.voter.to_owned()));
        VOTES.save(
            deps.storage,
            (challenge_id.clone(), vote.voter.to_owned()),
            v,
        )?;

        return Ok(Response::new().add_attribute("action", "count_votes"));
    } else {
        Err(AppError::VoterNotFound {})
    }
}

fn charge_penalty(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    deps.api.addr_validate(info.sender.as_ref())?;
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    let friends = CHALLENGE_FRIENDS.load(deps.storage, challenge_id.clone())?;

    let bank = app.bank(deps.as_ref());
    let executor = app.executor(deps.as_ref());

    match challenge.collateral {
        Penalty::FixedAmount { asset } => {
            let num_friends = friends.len() as u128;
            if num_friends == 0 {
                return Err(AppError::Std(StdError::generic_err(
                    "No friends found for the challenge.",
                )));
            }

            // Calculate each friend's share
            let amount_per_friend = asset.amount.u128() / num_friends;
            let asset_per_friend = OfferAsset {
                name: asset.name,
                amount: Uint128::from(amount_per_friend),
            };

            // Create a transfer action for each friend
            let transfer_actions: Result<Vec<_>, _> = friends
                .into_iter()
                .map(|friend| bank.transfer(vec![asset_per_friend.clone()], &friend.address))
                .collect();

            let transfer_msgs = executor.execute(transfer_actions?);

            Ok(Response::new()
                .add_messages(transfer_msgs)
                .add_attribute("total_amount", asset.amount.to_string())
                .add_attribute("action", "charge_fixed_amount_penalty"))
        }
        Penalty::Daily {
            asset: _,
            split_between_friends: _,
        } => {
            // Not sure what the exact implementation should be here.
            // Is it that for this variant we want to only charge_penalty at the end of the
            // challenge? If so how do we determine when the challenge has come to an end?
            //let _transfer_action = bank.transfer(vec![asset], &admin_address)?;
            Ok(Response::new().add_attribute("action", "charge_daily_penalty"))
        }
    }
}
