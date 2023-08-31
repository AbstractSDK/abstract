use crate::error::AppError;
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::contract::{AppResult, ChallengeApp};
use abstract_sdk::prelude::*;
use chrono::{Datelike, NaiveDateTime};

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
        ChallengeExecuteMsg::VetoVote {
            voter,
            challenge_id,
        } => veto_vote(deps, info, env, &app, challenge_id, voter),
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
        "update_challenge",
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
            return Err(AppError::Std(StdError::generic_err(
                "Friend already added for this challenge",
            )));
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
        .map_or(false, |check_in| {
            check_in.last_checked_in == Some(now.clone())
        })
    {
        return Err(AppError::AlreadyCheckedIn {});
    }

    let _blocks_per_day = 1440; // dummy value, check this
    let next_check_in_block = env.block.height + 10;

    let check_in = CheckIn {
        last_checked_in: Some(now),
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
        .map_or(false, |check_in| check_in.last_checked_in == Some(now))
    {
        return Err(AppError::AlreadyCheckedIn {});
    }

    let vote = vote.optimisitc();

    // Load existing votes for the current block height or initialize an empty list if none exist
    let mut votes_for_block = VOTES
        .load(deps.storage, challenge_id.to_owned())
        .unwrap_or_else(|_| Vec::new());

    // check if final_vote.voter already exists in votes_for_block
    if votes_for_block.iter().any(|v| v.voter == vote.voter) {
        return Err(AppError::AlreadyVoted {});
    }

    // Append the new vote and save them to storage
    votes_for_block.push(vote);
    VOTES.save(deps.storage, challenge_id.to_owned(), &votes_for_block)?;

    Ok(Response::new().add_attribute("action", "cast_vote"))
}

fn count_votes(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    let votes_for_challenge = VOTES
        .load(deps.storage, challenge_id.clone())
        .unwrap_or_else(|_| Vec::new());

    let any_false_vote = votes_for_challenge
        .iter()
        .any(|v| v.approval == Some(false));

    if any_false_vote {
        return charge_penalty(deps, info, app, challenge_id);
    }

    Ok(Response::new().add_attribute("action", "count_votes"))
}

fn veto_vote(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    app: &ChallengeApp,
    challenge_id: u64,
    voter: String,
) -> AppResult {
    if info.sender.to_string() != ADMIN.load(deps.storage)? {
        return Err(AppError::Std(StdError::generic_err(
            "Only the admin can veto a vote",
        )));
    }
    let votes = VOTES
        .load(deps.storage, challenge_id.clone())
        .unwrap_or_else(|_| Vec::new());

    // find the voter in the votes_for_challenge
    let disputed = votes.iter().find(|v| v.voter == voter).ok_or_else(|| {
        AppError::Std(StdError::generic_err(format!(
            "Voter {} not found for this challenge",
            voter
        )))
    })?;

    let mut vetoed_votes = votes.clone();
    //
    //remove the disputed from the votes Vec
    vetoed_votes.retain(|v| v.voter != disputed.voter);

    VOTES.remove(deps.storage, challenge_id.clone());
    VOTES.save(deps.storage, challenge_id.clone(), &vetoed_votes)?;

    let mut disputed = disputed.clone();
    // set the vote the opposite to what it currently is
    disputed.approval = Some(!disputed.approval.unwrap());

    Ok(Response::new().add_attribute("action", "count_votes"))
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
            asset,
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

fn date_from_block(env: &Env) -> String {
    // Convert the block's timestamp to NaiveDateTime
    let seconds = env.block.time.seconds();
    let nano_seconds = env.block.time.subsec_nanos();
    let dt = NaiveDateTime::from_timestamp_opt(seconds as i64, nano_seconds as u32);

    format!(
        "{year}-{month}-{day}",
        year = dt.unwrap().year(),
        month = dt.unwrap().month(),
        day = dt.unwrap().day()
    )
}
