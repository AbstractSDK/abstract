use crate::error::AppError;
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Timestamp, Uint128};

use crate::contract::{AppResult, ChallengeApp};
use abstract_sdk::prelude::*;

use crate::msg::ChallengeExecuteMsg;
use crate::state::{
    ChallengeEntry, ChallengeEntryUpdate, ChallengeStatus, CheckIn, CheckInStatus, EndType, Friend,
    UpdateFriendsOpKind, Vote, CHALLENGE_FRIENDS, CHALLENGE_LIST, CHALLENGE_VOTES, DAILY_CHECK_INS,
    NEXT_ID, VOTES,
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
        } => update_challenge(deps, info, app, challenge_id, challenge),
        ChallengeExecuteMsg::CancelChallenge { challenge_id } => {
            cancel_challenge(deps, info, &app, challenge_id)
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
            cast_vote(deps, env, &app, vote, challenge_id)
        }
    }
}

/// Create new challenge
fn create_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    mut challenge: ChallengeEntry,
) -> AppResult {
    // Only the admin should be able to create a challenge.
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    //check that the challenge status is ChallengeStatus::Uninitialized
    if challenge.status != ChallengeStatus::Uninitialized {
        return Err(AppError::WrongChallengeStatus {});
    }

    let mut challenge = challenge.set_end_timestamp(&env)?;
    challenge.set_total_check_ins(&env)?;

    // Generate the challenge id and update the status
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    challenge.status = ChallengeStatus::Active;
    CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

    // Create the initial check_in entry
    DAILY_CHECK_INS.save(
        deps.storage,
        challenge_id.clone(),
        &vec![CheckIn::default_from(&env)],
    )?;

    // Create the initial challenge_votes entry
    CHALLENGE_VOTES.save(deps.storage, challenge_id.clone(), &Vec::new())?;

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
        .may_load(deps.storage, challenge_id.clone())
        .map_err(|_| AppError::NotFound {})?
        .ok_or_else(|| {
            AppError::Std(StdError::generic_err(format!(
                "Challenge with id {} not found",
                challenge_id
            )))
        })?;

    if loaded_challenge.status != ChallengeStatus::Active {
        return Err(AppError::WrongChallengeStatus {});
    }

    if let Some(name) = new_challenge.name {
        loaded_challenge.name = name;
    }
    if let Some(collateral) = new_challenge.collateral {
        loaded_challenge.collateral = collateral;
    }
    if let Some(description) = new_challenge.description {
        loaded_challenge.description = description;
    }
    if let Some(end) = new_challenge.end {
        loaded_challenge.end = EndType::ExactTime(end);
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
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let mut challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    if challenge.status != ChallengeStatus::Active {
        return Err(AppError::WrongChallengeStatus {});
    }

    challenge.status = ChallengeStatus::Cancelled;
    CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

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
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    if challenge.status != ChallengeStatus::Active {
        return Err(AppError::WrongChallengeStatus {});
    }

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
    let mut challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    let now = Timestamp::from(env.block.time);

    // If the challenge has ended, we set the status to Over and return
    if now > challenge.get_end_timestamp()? {
        match challenge.status {
            ChallengeStatus::Active => {
                challenge.status = ChallengeStatus::Over;
                CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;
                return Ok(Response::new()
                    .add_attribute("action", "check_in")
                    .add_attribute(
                        "message",
                        "Challenge has ended. You can no longer register a daily check in.",
                    ));
            }
            _ => {
                return Err(AppError::Std(StdError::generic_err(format!(
                    "Challenge has ended. Challenge is {:?} current block height is {}",
                    challenge, env.block.height
                ))));
            }
        }
    }

    let mut check_ins = DAILY_CHECK_INS.load(deps.storage, challenge_id.clone())?;
    let check_in = check_ins.last().unwrap();
    let now = Timestamp::from(env.block.time);
    let next = Timestamp::from_seconds(env.block.time.seconds() + 60 * 60 * 24);

    match now {
        now if now == check_in.last => {
            return Err(AppError::AlreadyCheckedIn {});
        }

        // The admin has missed the deadline for checking in, they are given a strike.
        // The contract manually sets the next check in time.
        now if now >= check_in.next => {
            for strike in challenge.admin_strikes.iter_mut() {
                if !*strike == false {
                    *strike = true;
                    break;
                }
            }
            CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

            // If the admin has 3 strikes, the challenge is cancelled.
            if challenge.admin_strikes.iter().all(|strike| *strike) {
                return cancel_challenge(deps, info, app, challenge_id);
            }

            let check_in = CheckIn {
                last: check_in.last,
                next,
                metadata,
                status: CheckInStatus::MissedCheckIn,
                tally_result: None,
            };
            check_ins.push(check_in);
            DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_ins)?;

            return Ok(Response::new()
                .add_attribute("action", "check_in")
                .add_attribute(
                    "message",
                    "You missed the deadline for checking in. You have been given a strike.",
                ));
        }

        // The admin is checking in on time, so we can proceeed.
        now if now < check_in.next => {
            let check_in = CheckIn {
                last: now,
                next,
                metadata,
                status: CheckInStatus::CheckedInNotYetVoted,
                tally_result: None,
            };

            check_ins.push(check_in);
            DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_ins)?;
            return Ok(Response::new().add_attribute("action", "check_in"));
        }
        _ => {
            return Err(AppError::Std(StdError::generic_err(
                "Something went wrong with the check in.",
            )));
        }
    }
}

fn cast_vote(
    deps: DepsMut,
    env: Env,
    app: &ChallengeApp,
    vote: Vote<String>,
    challenge_id: u64,
) -> AppResult {
    let mut vote = vote.check(deps.as_ref())?.optimistic();

    let mut check_ins = DAILY_CHECK_INS.load(deps.storage, challenge_id.clone())?;
    // We can unwrap because there will always be atleast one element in the vector
    let check_in = check_ins.last_mut().unwrap();

    if check_in.status != CheckInStatus::CheckedInNotYetVoted
        && check_in.status != CheckInStatus::VotedNotYetTallied
    {
        return Err(AppError::Std(StdError::generic_err(format!(
            "Wrong check in status {:?} for casting vote",
            check_in.status
        ))));
    }

    // Check if the voter has already voted
    if VOTES
        .may_load(deps.storage, (check_in.last.nanos(), vote.voter.to_owned()))
        .map_or(false, |votes| votes.iter().any(|v| v.voter == vote.voter))
    {
        return Err(AppError::AlreadyVoted {});
    }

    VOTES.save(
        deps.storage,
        (check_in.last.nanos(), vote.voter.to_owned()),
        &vote,
    )?;

    let mut challenge_votes = CHALLENGE_VOTES.load(deps.storage, challenge_id.clone())?;
    vote.for_check_in = Some(check_in.last);
    challenge_votes.push(vote);
    CHALLENGE_VOTES.save(deps.storage, challenge_id.clone(), &challenge_votes)?;

    check_in.status = CheckInStatus::VotedNotYetTallied;
    DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_ins)?;

    // If all friends have voted, we tally the votes.
    if CHALLENGE_VOTES
        .load(deps.storage, challenge_id.clone())?
        .len()
        == CHALLENGE_FRIENDS.load(deps.storage, challenge_id)?.len()
    {
        return tally_votes_for_check_in(deps, env, app, challenge_id);
    }

    Ok(Response::new().add_attribute("action", "cast_vote"))
}

fn tally_votes_for_check_in(
    deps: DepsMut,
    _env: Env,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    let mut check_ins = DAILY_CHECK_INS.load(deps.storage, challenge_id.clone())?;
    let check_in = check_ins.last_mut().unwrap();
    if check_in.status != CheckInStatus::VotedNotYetTallied {
        return Err(AppError::WrongCheckInStatus {});
    }

    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    let votes = CHALLENGE_VOTES.load(deps.storage, challenge_id)?;

    // check for any false votes on check_ins that match the vote timestamps
    let any_false_vote = votes
        .iter()
        .filter(|&vote| {
            vote.for_check_in
                .map_or(false, |timestamp| timestamp == check_in.last)
        })
        .any(|v| v.approval == Some(false));

    check_in.status = CheckInStatus::VotedAndTallied;

    if any_false_vote {
        check_in.tally_result = Some(false);
        DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_ins)?;
        return charge_penalty(deps, &app, challenge_id);
    } else {
        check_in.tally_result = Some(true);
        DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_ins)?;
        CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

        return Ok(Response::new()
            .add_attribute("action", "tally_vote")
            .add_attribute(
                "message",
                "All votes were positive. ChallengeStatus has been set to OverAndCompleted.",
            ));
    }
}

fn charge_penalty(deps: DepsMut, app: &ChallengeApp, challenge_id: u64) -> AppResult {
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    let friends = CHALLENGE_FRIENDS.load(deps.storage, challenge_id.clone())?;

    let num_friends = friends.len() as u128;
    if num_friends == 0 {
        return Err(AppError::Std(StdError::generic_err(
            "No friends found for the challenge.",
        )));
    }

    let amount_per_friend =
        (challenge.collateral.amount.u128() / challenge.total_check_ins.unwrap()) / num_friends;

    let reaminder = challenge.collateral.amount.u128() % num_friends;

    let asset_per_friend = OfferAsset {
        name: challenge.collateral.name,
        amount: Uint128::from(amount_per_friend),
    };

    let bank = app.bank(deps.as_ref());
    let executor = app.executor(deps.as_ref());

    // Create a transfer action for each friend
    let transfer_actions: Result<Vec<_>, _> = friends
        .into_iter()
        .map(|friend| bank.transfer(vec![asset_per_friend.clone()], &friend.address))
        .collect();

    let transfer_msgs = executor.execute(transfer_actions?);

    Ok(Response::new()
        .add_messages(transfer_msgs)
        .add_attribute("total_amount", challenge.collateral.amount.to_string())
        .add_attribute("action", "charged penalty")
        .add_attribute("the remainder was", reaminder.to_string()))
}
