use crate::error::AppError;
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::contract::{AppResult, ChallengeApp};
use abstract_sdk::prelude::*;
use chrono::{Datelike, NaiveDateTime};

use crate::msg::ChallengeExecuteMsg;
use crate::state::{
    ChallengeEntry, CheckIn, Friend, Penalty, Vote, CHALLENGE_FRIENDS, CHALLENGE_LIST,
    DAILY_CHECK_INS, NEXT_ID, VOTES,
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
        ChallengeExecuteMsg::AddFriendForChallenge {
            challenge_id,
            friend_name,
            friend_address,
        } => add_friend_for_challenge(
            deps,
            info,
            &app,
            challenge_id,
            &friend_name,
            &friend_address,
        ),
        ChallengeExecuteMsg::RemoveFriendForChallenge {
            challenge_id,
            friend_address,
        } => remove_friend_from_challenge(deps, info, &app, challenge_id, friend_address),
        ChallengeExecuteMsg::AddFriendsForChallenge {
            challenge_id,
            friends,
        } => add_friends_for_challenge(deps, info, &app, challenge_id, friends),
        ChallengeExecuteMsg::DailyCheckIn {
            challenge_id,
            metadata,
        } => daily_check_in(deps, env, info, &app, challenge_id, metadata),
        ChallengeExecuteMsg::CastVote { vote, challenge_id } => {
            cast_vote(deps, env, info, &app, vote, &challenge_id)
        }
        ChallengeExecuteMsg::CountVotes { challenge_id } => {
            count_votes(deps, info, env, &app, challenge_id)
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
    let id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    let challenge_id = format!("challenge_{id}");

    CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id),
        "create_challenge",
    ))
}

/// Update an existing challenge  
fn update_challenge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    challenge_id: String,
    challenge: ChallengeEntry,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // will return an error if the challenge doesn't exist
    CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id),
        "update_challenge",
    ))
}

fn cancel_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: ChallengeApp,
    challenge_id: String,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    CHALLENGE_LIST.remove(deps.storage, challenge_id.clone());

    Ok(Response::new().add_attribute("action", "cancel_challenge"))
}

fn add_friend_for_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: String,
    friend_name: &String,
    friend_address: &String,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut friends_for_challenge = CHALLENGE_FRIENDS
        .may_load(deps.storage, challenge_id.clone())?
        .unwrap_or_else(Vec::new);

    if friends_for_challenge
        .iter()
        .any(|f| &f.address == friend_address)
    {
        return Err(AppError::Std(StdError::generic_err(
            "Friend already added for this challenge",
        )));
    }

    let friend_address = deps.api.addr_validate(&friend_address)?;

    let friend = Friend {
        address: friend_address.clone(),
        name: friend_name.clone(),
    };

    friends_for_challenge.push(friend);
    CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &friends_for_challenge)?;

    Ok(Response::new().add_attribute("action", "add_friend"))
}

pub fn remove_friend_from_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: String,
    friend_address: String,
) -> AppResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut friends_for_challenge = CHALLENGE_FRIENDS
        .may_load(deps.storage, challenge_id.clone())?
        .unwrap_or_else(Vec::new);

    let friend_index = friends_for_challenge
        .iter()
        .position(|f| f.address == friend_address);

    match friend_index {
        Some(index) => {
            friends_for_challenge.remove(index);
            CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &friends_for_challenge)?;
        }
        None => {
            return Err(AppError::Std(StdError::generic_err(
                "Friend not found for this challenge",
            )));
        }
    }

    Ok(Response::new().add_attribute("action", "remove_friend"))
}

fn add_friends_for_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: String,
    friends: Vec<Friend>,
) -> AppResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut existing_friends = CHALLENGE_FRIENDS
        .may_load(deps.storage, challenge_id.clone())?
        .unwrap_or_else(Vec::new);

    for friend in &friends {
        if existing_friends
            .iter()
            .any(|f| &f.address == &friend.address)
        {
            return Err(AppError::Std(StdError::generic_err(
                "Friend already added for this challenge",
            )));
        }
    }

    existing_friends.extend(friends.into_iter());
    CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &existing_friends)?;
    Ok(Response::new().add_attribute("action", "add_friends"))
}

fn daily_check_in(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: String,
    metadata: Option<String>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let today = date_from_block(&env);

    // Check if Admin has already checked in today
    if DAILY_CHECK_INS
        .load(deps.storage, today.clone())
        .map_or(false, |check_in| {
            check_in.last_checked_in.as_deref() == Some(&today)
        })
    {
        return Err(AppError::AlreadyCheckedIn {});
    }

    let _blocks_per_day = 1440; // dummy value, check this
    let next_check_in_block = env.block.height + 10;

    let check_in = CheckIn {
        last_checked_in: Some(today.clone()),
        next_check_in_by: next_check_in_block,
        metadata,
    };

    DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_in)?;
    Ok(Response::new().add_attribute("action", "check_in"))
}

fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _app: &ChallengeApp,
    vote: Option<bool>,
    challenge_id: &String,
) -> AppResult {
    let today = date_from_block(&env);

    // If Admin checked in today, friends can't vote
    if DAILY_CHECK_INS
        .load(deps.storage, today.clone())
        .map_or(false, |check_in| {
            check_in.last_checked_in.as_deref() == Some(&today)
        })
    {
        return Err(AppError::AlreadyCheckedIn {});
    }

    // If the vote is None, default to true (meaning we assume Admin fulfilled his challenge)
    let final_vote = vote.unwrap_or(true);

    // Construct the vote
    let vote_entry = Vote {
        voter: info.sender.to_string(),
        vote: Some(final_vote),
        challenge_id: challenge_id.clone(),
    };

    // Load existing votes for the current block height or initialize an empty list if none exist
    let mut votes_for_block = VOTES
        .load(deps.storage, challenge_id.clone())
        .unwrap_or_else(|_| Vec::new());

    // Append the new vote
    votes_for_block.push(vote_entry);

    // Save the updated votes
    VOTES.save(deps.storage, challenge_id.clone(), &votes_for_block)?;

    Ok(Response::new().add_attribute("action", "cast_vote"))
}

fn count_votes(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    app: &ChallengeApp,
    challenge_id: String,
) -> AppResult {
    let votes_for_challenge = VOTES
        .load(deps.storage, challenge_id.clone())
        .unwrap_or_else(|_| Vec::new());

    let any_false_vote = votes_for_challenge.iter().any(|v| v.vote == Some(false));
    println!("any_false_vote: {}", any_false_vote);
    if any_false_vote {
        return charge_penalty(deps, info, app, challenge_id);
    }

    Ok(Response::new().add_attribute("action", "count_votes"))
}

fn charge_penalty(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: String,
) -> AppResult {
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    let friends = CHALLENGE_FRIENDS.load(deps.storage, challenge_id.clone())?;

    let admin_address = deps.api.addr_validate(&info.sender.to_string())?;
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

            return Ok(Response::new()
                .add_messages(transfer_msgs)
                .add_attribute("action", "charge_fixed_amount_penalty"));
        }
        Penalty::Daily {
            asset,
            split_between_friends: _,
        } => {
            // Not sure what the exact implementation should be here.
            // Is it that for this variant we want to only charge_penalty at the end of the
            // challenge? If so how do we determine when the challenge has come to an end?
            let _transfer_action = bank.transfer(vec![asset], &admin_address)?;
            return Ok(Response::new().add_attribute("action", "charge_daily_penalty"));
        }
    }
}

fn date_from_block(env: &Env) -> String {
    // Convert the block's timestamp to NaiveDateTime
    let seconds = env.block.time.seconds();
    let nano_seconds = env.block.time.subsec_nanos();
    let dt = NaiveDateTime::from_timestamp(seconds as i64, nano_seconds as u32);

    // Format the date using the NaiveDateTime object
    format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day())
}
