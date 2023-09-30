use crate::error::AppError;
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Timestamp, Uint128};

use crate::contract::{AppResult, ChallengeApp};
use abstract_sdk::prelude::*;

use crate::msg::{ChallengeExecuteMsg, ChallengeRequest};
use crate::state::{
    ChallengeEntry, ChallengeEntryUpdate, ChallengeStatus, CheckIn, CheckInStatus, Friend,
    UpdateFriendsOpKind, Vote, CHALLENGE_FRIENDS, CHALLENGE_LIST, CHALLENGE_VOTES, DAILY_CHECK_INS,
    DAY, NEXT_ID, VOTES,
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
    challenge_req: ChallengeRequest,
) -> AppResult {
    // Only the admin should be able to create a challenge.
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut challenge = ChallengeEntry::new(challenge_req);

    //check that the challenge status is ChallengeStatus::Uninitialized
    if challenge.status != ChallengeStatus::Uninitialized {
        return Err(AppError::WrongChallengeStatus {});
    }

    // Generate the challenge id and update the status
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    challenge.status = ChallengeStatus::Active;
    CHALLENGE_LIST.save(deps.storage, challenge_id, &challenge)?;

    // Create the initial check_in entry
    DAILY_CHECK_INS.save(
        deps.storage,
        challenge_id,
        &vec![CheckIn::default_from(&env)],
    )?;

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
        .ok_or(AppError::NotFound {})?;

    if loaded_challenge.status != ChallengeStatus::Active {
        return Err(AppError::WrongChallengeStatus {});
    }

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
    if challenge.status != ChallengeStatus::Active {
        return Err(AppError::WrongChallengeStatus {});
    }

    challenge.status = ChallengeStatus::Cancelled;
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

fn daily_check_in(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    metadata: Option<String>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let mut challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;

    // If the challenge has ended, we set the status to Over and return
    if challenge.end.is_expired(&env.block) {
        match challenge.status {
            ChallengeStatus::Active => {
                challenge.status = ChallengeStatus::Over;
                CHALLENGE_LIST.save(deps.storage, challenge_id, &challenge)?;
                return Ok(app.tag_response(
                    Response::new().add_attribute(
                        "message",
                        "Challenge has ended. You can no longer register a daily check in.",
                    ),
                    "daily check in",
                ));
            }
            _ => {
                return Err(AppError::Std(StdError::generic_err(format!(
                    "Challenge has ended. Challenge end is {:?} current block is {:?}",
                    challenge.end, env.block
                ))));
            }
        }
    }

    let mut check_ins = DAILY_CHECK_INS.load(deps.storage, challenge_id)?;
    let check_in = check_ins.last().unwrap();
    let next = Timestamp::from_seconds(env.block.time.seconds() + DAY);

    match env.block.time {
        now if now == check_in.last => Err(AppError::AlreadyCheckedIn {}),

        // The admin has missed the deadline for checking in, they are given a strike.
        // The contract manually sets the next check in time.
        now if now >= check_in.next => {
            challenge.admin_strikes.num_strikes += 1;
            CHALLENGE_LIST.save(deps.storage, challenge_id, &challenge)?;

            // If the admin's strikes reach the limit, cancel the challenge
            if challenge.admin_strikes.num_strikes >= challenge.admin_strikes.limit {
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

            Ok(app.tag_response(
                Response::new().add_attribute(
                    "message",
                    "You missed the daily check in, you have been given a strike",
                ),
                "daily check in",
            ))
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

            Ok(app.tag_response(
                Response::new().add_attribute("action", "check_in"),
                "daily check in",
            ))
        }
        _ => Err(AppError::Std(StdError::generic_err(
            "Something went wrong with the check in.",
        ))),
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

    let mut check_ins = DAILY_CHECK_INS.load(deps.storage, challenge_id)?;
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
        .may_load(
            deps.storage,
            (challenge_id, check_in.last.nanos(), vote.voter.to_owned()),
        )
        .map_or(false, |votes| votes.iter().any(|v| v.voter == vote.voter))
    {
        return Err(AppError::AlreadyVoted {});
    }

    VOTES.save(
        deps.storage,
        (challenge_id, check_in.last.nanos(), vote.voter.to_owned()),
        &vote,
    )?;

    let mut challenge_votes = CHALLENGE_VOTES.load(deps.storage, challenge_id)?;
    vote.for_check_in = Some(check_in.last);
    challenge_votes.push(vote);
    CHALLENGE_VOTES.save(deps.storage, challenge_id, &challenge_votes)?;

    check_in.status = CheckInStatus::VotedNotYetTallied;
    DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_ins)?;

    // If all friends have voted, we tally the votes.
    if CHALLENGE_VOTES.load(deps.storage, challenge_id)?.len()
        == CHALLENGE_FRIENDS.load(deps.storage, challenge_id)?.len()
    {
        return tally_votes_for_check_in(deps, env, app, challenge_id);
    }

    Ok(app.tag_response(
        Response::new().add_attribute("action", "cast_vote"),
        "cast_vote",
    ))
}

fn tally_votes_for_check_in(
    deps: DepsMut,
    _env: Env,
    app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    let mut check_ins = DAILY_CHECK_INS.load(deps.storage, challenge_id)?;
    let check_in = check_ins.last_mut().unwrap();
    if check_in.status != CheckInStatus::VotedNotYetTallied {
        return Err(AppError::WrongCheckInStatus {});
    }

    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
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

        charge_penalty(deps, app, challenge_id)
    } else {
        check_in.tally_result = Some(true);
        DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_ins)?;
        CHALLENGE_LIST.save(deps.storage, challenge_id, &challenge)?;

        Ok(app.tag_response(
            Response::new().add_attribute(
                "message",
                "All votes were positive. ChallengeStatus has been set to OverAndCompleted.",
            ),
            "tally_votes_for_check_in",
        ))
    }
}

fn charge_penalty(deps: DepsMut, app: &ChallengeApp, challenge_id: u64) -> AppResult {
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let friends = CHALLENGE_FRIENDS.load(deps.storage, challenge_id)?;

    let num_friends = friends.len() as u128;
    if num_friends == 0 {
        return Err(AppError::Std(StdError::generic_err(
            "No friends found for the challenge.",
        )));
    }

    let compute_amount_per_friend = || -> Result<u128, AppError> {
        if num_friends == 0 {
            return Err(AppError::Std(StdError::generic_err(format!(
                "Cannot compute amount per friend. num_friends: {}",
                num_friends
            ))));
        }
        Ok(challenge.strike_amount)
    };

    let reaminder = challenge.collateral.amount.u128() % num_friends;

    let asset_per_friend = OfferAsset {
        name: challenge.collateral.name,
        amount: Uint128::from(compute_amount_per_friend()?),
    };

    let bank = app.bank(deps.as_ref());
    let executor = app.executor(deps.as_ref());

    // Create a transfer action for each friend
    let transfer_actions: Result<Vec<_>, _> = friends
        .into_iter()
        .map(|friend| bank.transfer(vec![asset_per_friend.clone()], &friend.address))
        .collect();

    let transfer_msg = executor.execute(transfer_actions?);

    Ok(app
        .tag_response(
            Response::new().add_attribute(
                "message",
                "All votes were negative. ChallengeStatus has been set to OverAndCompleted.",
            ),
            "charge_penalty",
        )
        .add_messages(transfer_msg)
        .add_attribute("remainder was", reaminder.to_string()))
}
