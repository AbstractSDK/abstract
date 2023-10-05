use crate::error::AppError;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError};

use crate::contract::{AppResult, ChallengeApp};
// use abstract_sdk::prelude::*;

use crate::msg::{ChallengeExecuteMsg, ChallengeRequest};
use crate::state::{
    ChallengeEntry, ChallengeEntryUpdate, ChallengeStatus, Friend, UpdateFriendsOpKind, Vote,
    CHALLENGE_FRIENDS, CHALLENGE_LIST, CHALLENGE_VOTES, NEXT_ID, VOTES,
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
        } => update_challenge(deps, info, app, challenge_id, challenge),
        ChallengeExecuteMsg::CancelChallenge { challenge_id } => {
            cancel_challenge(deps, info, &app, challenge_id)
        }
        ChallengeExecuteMsg::UpdateFriendsForChallenge {
            challenge_id,
            friends,
            op_kind,
        } => update_friends_for_challenge(deps, info, &app, challenge_id, friends, op_kind),
        ChallengeExecuteMsg::CastVote { vote, challenge_id } => {
            cast_vote(deps, env, &app, vote, challenge_id)
        }
    }
}

/// Create new challenge
fn create_challenge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    challenge_req: ChallengeRequest,
) -> AppResult {
    // Only the admin should be able to create a challenge.
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let challenge = ChallengeEntry::new(challenge_req);

    // Generate the challenge id and update the status
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    CHALLENGE_LIST.save(deps.storage, challenge_id, &challenge)?;

    // Create the initial challenge_votes entry
    CHALLENGE_VOTES.save(deps.storage, challenge_id, &Vec::new())?;

    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "create_challenge",
    ))
}

fn update_challenge(
    deps: DepsMut,
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
    loaded_challenge.status.assert_active()?;

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
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let mut challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    challenge.status.assert_active()?;

    challenge.status = ChallengeStatus::Cancelled {};
    CHALLENGE_LIST.save(deps.storage, challenge_id, &challenge)?;

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
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    challenge.status.assert_active()?;

    match op_kind {
        UpdateFriendsOpKind::Add {} => {
            add_friends_for_challenge(deps, info, app, challenge_id, friends)
        }
        UpdateFriendsOpKind::Remove {} => {
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
        .may_load(deps.storage, challenge_id)?
        .unwrap_or_default();

    for friend in &friends {
        if existing_friends.iter().any(|f| f.address == friend.address) {
            return Err(AppError::AlreadyAdded {});
        }
    }

    // validate the String addresses and convert them to Addr
    // before saving
    let friends: Result<Vec<Friend<Addr>>, _> = friends
        .into_iter()
        .map(|friend| friend.check(deps.as_ref()))
        .collect();

    match friends {
        Ok(friends) => {
            existing_friends.extend(friends);
            CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &existing_friends)?;

            Ok(app.tag_response(
                Response::new().add_attribute("challenge_id", challenge_id.to_string()),
                "add_friends",
            ))
        }
        Err(err) => Err(AppError::Std(StdError::generic_err(format!(
            "Error adding friends: {:?}",
            err
        )))),
    }
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
        .may_load(deps.storage, challenge_id)?
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
    Ok(app.tag_response(
        Response::new().add_attribute("challenge_id", challenge_id.to_string()),
        "remove_friends",
    ))
}

fn cast_vote(
    deps: DepsMut,
    _env: Env,
    app: &ChallengeApp,
    vote: Vote<String>,
    challenge_id: u64,
) -> AppResult {
    let vote = vote.check(deps.as_ref())?.optimistic();

    // Check if the voter has already voted
    if VOTES
        .may_load(deps.storage, (challenge_id, vote.voter.to_owned()))
        .map_or(false, |votes| votes.iter().any(|v| v.voter == vote.voter))
    {
        return Err(AppError::AlreadyVoted {});
    }

    VOTES.save(deps.storage, (challenge_id, vote.voter.to_owned()), &vote)?;

    let mut challenge_votes = CHALLENGE_VOTES.load(deps.storage, challenge_id)?;
    challenge_votes.push(vote);
    CHALLENGE_VOTES.save(deps.storage, challenge_id, &challenge_votes)?;

    Ok(app.tag_response(
        Response::new().add_attribute("action", "cast_vote"),
        "cast_vote",
    ))
}

// fn charge_penalty(deps: DepsMut, app: &ChallengeApp, challenge_id: u64) -> AppResult {
//     let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
//     let friends = CHALLENGE_FRIENDS.load(deps.storage, challenge_id)?;

//     let num_friends = friends.len() as u128;
//     if num_friends == 0 {
//         return Err(AppError::ZeroFriends {});
//     }

//     let (amount_per_friend, remainder) = match challenge.strike_strategy {
//         crate::state::StrikeStrategy::Split(amount) => (
//             Uint128::new(amount.u128() / num_friends),
//             amount.u128() % num_friends,
//         ),
//         crate::state::StrikeStrategy::PerFriend(amount) => (amount, 0),
//     };

//     let asset_per_friend = OfferAsset {
//         name: challenge.strike_asset,
//         amount: amount_per_friend,
//     };

//     let bank = app.bank(deps.as_ref());
//     let executor = app.executor(deps.as_ref());

//     // Create a transfer action for each friend
//     let transfer_actions: Result<Vec<_>, _> = friends
//         .into_iter()
//         .map(|friend| bank.transfer(vec![asset_per_friend.clone()], &friend.address))
//         .collect();

//     let transfer_msg = executor.execute(transfer_actions?);

//     Ok(app
//         .tag_response(
//             Response::new().add_attribute(
//                 "message",
//                 "All votes were negative. ChallengeStatus has been set to OverAndCompleted.",
//             ),
//             "charge_penalty",
//         )
//         .add_messages(transfer_msg)
//         .add_attribute("remainder", remainder.to_string()))
// }
