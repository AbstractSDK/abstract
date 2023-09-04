use crate::error::AppError;
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Timestamp, Uint128};

use crate::contract::{AppResult, ChallengeApp};
use abstract_sdk::prelude::*;

use crate::msg::ChallengeExecuteMsg;
use crate::state::{
    ChallengeEntry, ChallengeEntryUpdate, ChallengeStatus, CheckIn, CheckInStatus, DurationChoice,
    EndType, Friend, Penalty, UpdateFriendsOpKind, Vote, ADMIN, CHALLENGE_FRIENDS, CHALLENGE_LIST,
    CHALLENGE_VOTES, DAILY_CHECK_INS, NEXT_ID, VOTES,
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
            cast_vote(deps, env, info, &app, vote, challenge_id)
        }

        ChallengeExecuteMsg::TallyVotes { challenge_id } => tally_votes(deps, env, challenge_id),
        ChallengeExecuteMsg::VetoVote { vote, challenge_id } => {
            veto_vote(deps, info, env, challenge_id, vote)
        }
        ChallengeExecuteMsg::ChargePenalty { challenge_id } => {
            charge_penalty(deps, info, &app, challenge_id)
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

    // Generate the challenge id and update the status
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    challenge.status = ChallengeStatus::Active;
    CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

    // Create the initial check_in entry
    DAILY_CHECK_INS.save(
        deps.storage,
        challenge_id.clone(),
        &CheckIn::default_from(&env),
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
        loaded_challenge.end = EndType::Duration(DurationChoice::Week);
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

    // If the challenge has ended, we set the status to OverAndPending and return before checking
    // in.
    if now > challenge.get_end_timestamp()? {
        match challenge.status {
            ChallengeStatus::Active => {
                challenge.status = ChallengeStatus::OverAndPending;
                CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;
                return Ok(Response::new()
                    .add_attribute("action", "check_in")
                    .add_attribute(
                        "message",
                        "Challenge has ended. You can no longer register a daily check in. 
                         ChallengeStatus has been set to OverAndPending.",
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

    let check_in = DAILY_CHECK_INS.may_load(deps.storage, challenge_id.clone())?;

    if let Some(check_in) = check_in {
        let now = Timestamp::from(env.block.time);
        let next_check_in_by = Timestamp::from_seconds(env.block.time.seconds() + 60 * 60 * 24);

        match now {
            now if now == check_in.last_checked_in => {
                return Err(AppError::AlreadyCheckedIn {});
            }

            // The admin has missed the deadline for checking in, they are given a strike.
            // The contract manually sets the next check in time.
            now if now >= check_in.next_check_in_by => {
                for strike in challenge.admin_strikes.iter_mut() {
                    if !*strike {
                        *strike = true;
                        break;
                    }
                }
                CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

                // If the admin has 3 strikes, the challenge is cancelled.
                if challenge.admin_strikes.iter().all(|strike| *strike) {
                    return cancel_challenge(deps, info, app, challenge_id);
                }

                let last = DAILY_CHECK_INS.load(deps.storage, challenge_id.clone())?;
                let check_in = CheckIn {
                    last_checked_in: last.last_checked_in,
                    next_check_in_by,
                    metadata,
                    status: CheckInStatus::MissedCheckIn,
                    tally_result: None,
                };
                DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_in)?;

                return Ok(Response::new()
                    .add_attribute("action", "check_in")
                    .add_attribute(
                        "message",
                        "You missed the deadline for checking in. You have been given a strike.",
                    ));
            }

            // The admin is checking in on time, so we can proceeed.
            now if now < check_in.next_check_in_by => {
                let check_in = CheckIn {
                    last_checked_in: now,
                    next_check_in_by,
                    metadata,
                    status: CheckInStatus::CheckedInNotYetVoted,
                    tally_result: None,
                };

                DAILY_CHECK_INS.save(deps.storage, challenge_id, &check_in)?;
                return Ok(Response::new().add_attribute("action", "check_in"));
            }
            _ => {
                return Err(AppError::Std(StdError::generic_err(
                    "Something went wrong with the check in.",
                )));
            }
        }
    } else {
        return Err(AppError::Std(StdError::generic_err(
            "Check in not found for this challenge",
        )));
    }
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
    let vote = vote.optimisitc();

    let mut last_check_in = DAILY_CHECK_INS.load(deps.storage, challenge_id.clone())?;
    if last_check_in.status != CheckInStatus::CheckedInNotYetVoted {
        return Err(AppError::WrongCheckInStatus {});
    }

    // Check if the voter has already voted
    if VOTES
        .may_load(
            deps.storage,
            (challenge_id.to_owned(), vote.voter.to_owned()),
        )
        .map_or(false, |votes| votes.iter().any(|v| v.voter == vote.voter))
    {
        return Err(AppError::AlreadyVoted {});
    }

    VOTES.save(
        deps.storage,
        (challenge_id.to_owned(), vote.voter.to_owned()),
        &vote,
    )?;
    let mut challenge_votes = CHALLENGE_VOTES.load(deps.storage, challenge_id.clone())?;
    challenge_votes.push(vote);
    CHALLENGE_VOTES.save(deps.storage, challenge_id.clone(), &challenge_votes)?;

    last_check_in.status = CheckInStatus::VotedNotYetTallied;
    DAILY_CHECK_INS.save(deps.storage, challenge_id, &last_check_in)?;

    // If all friends have voted, we tally the votes.
    if CHALLENGE_VOTES
        .load(deps.storage, challenge_id.clone())?
        .len()
        == CHALLENGE_FRIENDS.load(deps.storage, challenge_id)?.len()
    {
        tally_votes(deps, env, challenge_id)?;
    }
    Ok(Response::new().add_attribute("action", "cast_vote"))
}

fn tally_votes(deps: DepsMut, _env: Env, challenge_id: u64) -> AppResult {
    let mut challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
    // if challenge.status != ChallengeStatus::OverAndPending {
    //     return Err(AppError::WrongChallengeStatus {});
    // }

    let last_check_in = DAILY_CHECK_INS.load(deps.storage, challenge_id.clone())?;
    if last_check_in.status != CheckInStatus::VotedNotYetTallied
        || last_check_in.status != CheckInStatus::Recount
    {
        return Err(AppError::WrongCheckInStatus {});
    }

    let votes = CHALLENGE_VOTES.load(deps.storage, challenge_id)?;

    let any_false_vote = votes.iter().any(|v| v.approval == Some(false));
    //@TODO only update ChallengeStatus if the challenge is OverAndPending
    // update check_in status
    // update check_in tally_result
    if any_false_vote {
        challenge.status = ChallengeStatus::OverAndFailed;
        CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;
        return Ok(Response::new()
            .add_attribute("action", "tally_vote")
            .add_attribute(
                "message",
                "At least one vote was negative. ChallengeStatus has been set to OverAndFailed.",
            ));
    } else {
        challenge.status = ChallengeStatus::OverAndCompleted;
        CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

        return Ok(Response::new()
            .add_attribute("action", "tally_vote")
            .add_attribute(
                "message",
                "All votes were positive. ChallengeStatus has been set to OverAndCompleted.",
            ));
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
    if challenge.status != ChallengeStatus::OverAndFailed {
        return Err(AppError::WrongChallengeStatus {});
    }
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
            let reaminder = asset.amount.u128() % num_friends;
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
                .add_attribute("action", "charge_fixed_amount_penalty")
                .add_attribute("remainder is", reaminder.to_string()))
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
    let fetched_vote = VOTES.may_load(
        deps.storage,
        (challenge_id.to_owned(), vote.voter.to_owned()),
    )?;

    // we set the vote the opposite to what it currently is
    if let Some(mut to_veto) = fetched_vote {
        to_veto.approval = Some(!to_veto.approval.unwrap());
        // Update state
        VOTES.save(
            deps.storage,
            (challenge_id.clone(), vote.voter.to_owned()),
            &to_veto,
        )?;
        CHALLENGE_VOTES.remove(deps.storage, challenge_id.clone());
        CHALLENGE_VOTES.save(deps.storage, challenge_id.clone(), &vec![to_veto])?;

        // Update the challenge status otherwise charge_penalty could be called
        // before the vetoed vote is tallied. tally_vote must be called again.
        let mut challenge = CHALLENGE_LIST.load(deps.storage, challenge_id.clone())?;
        challenge.status = ChallengeStatus::OverAndPending;
        CHALLENGE_LIST.save(deps.storage, challenge_id.clone(), &challenge)?;

        return Ok(Response::new().add_attribute("action", "veto vote"));
    } else {
        Err(AppError::VoterNotFound {})
    }
}
