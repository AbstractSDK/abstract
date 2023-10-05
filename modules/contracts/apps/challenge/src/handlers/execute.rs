use crate::error::AppError;
use abstract_core::objects::voting::Vote;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::contract::{AppResult, ChallengeApp};
// use abstract_sdk::prelude::*;

use crate::msg::{ChallengeExecuteMsg, ChallengeRequest};
use crate::state::{
    ChallengeEntry, ChallengeEntryUpdate, UpdateFriendsOpKind, CHALLENGE_LIST, NEXT_ID,
    SIMPLE_VOTING,
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
        ChallengeExecuteMsg::CastVote { vote, challenge_id } => {
            cast_vote(deps, env, info, &app, vote, challenge_id)
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

    let friends_validated = challenge_req
        .init_friends
        .iter()
        .map(|human| deps.api.addr_validate(human))
        .collect::<StdResult<Vec<Addr>>>()?;

    let end = challenge_req.duration.after(&env.block);
    let vote_id = SIMPLE_VOTING.new_vote(deps.storage, end, &friends_validated)?;

    let challenge = ChallengeEntry::new(challenge_req, vote_id);

    // Generate the challenge id and update the status
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
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
    let vote_id = loaded_challenge.current_vote_id;

    SIMPLE_VOTING
        .load_vote_info(deps.storage, vote_id)?
        .assert_ready_for_action(vote_id, &env.block)?;

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
    let mut challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let vote_id = challenge.current_vote_id;
    SIMPLE_VOTING.cancel_vote(deps.storage, &env.block, vote_id)?;

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
    friends: Vec<String>,
    op_kind: UpdateFriendsOpKind,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let friends_validated = friends
        .iter()
        .map(|human| deps.api.addr_validate(human))
        .collect::<StdResult<Vec<Addr>>>()?;

    let vote_id = challenge.current_vote_id;

    SIMPLE_VOTING
        .load_vote_info(deps.storage, vote_id)?
        .assert_ready_for_action(vote_id, &env.block)?;

    match op_kind {
        UpdateFriendsOpKind::Add {} => {
            SIMPLE_VOTING.add_voters(deps.storage, vote_id, &env.block, &friends_validated)?;
        }
        UpdateFriendsOpKind::Remove {} => {
            SIMPLE_VOTING.remove_voters(deps.storage, vote_id, &env.block, &friends_validated)?;
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
    let vote_id = challenge.current_vote_id;
    let vote_info =
        SIMPLE_VOTING.cast_vote(deps.storage, &env.block, vote_id, &info.sender, vote)?;

    Ok(app.tag_response(Response::new(), "cast_vote"))
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
