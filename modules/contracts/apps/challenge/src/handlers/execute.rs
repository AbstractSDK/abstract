use crate::error::AppError;
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use croncat_app::CronCatInterface;

use crate::contract::{AppResult, ChallengeApp};
use chrono::{Datelike, NaiveDateTime};

use crate::msg::{ChallengeExecuteMsg, Frequency};
use crate::state::{
    ChallengeEntry, CheckIn, Config, Friend, Vote, CHALLENGE_FRIENDS, CHALLENGE_LIST, CONFIG,
    DAILY_CHECKINS, NEXT_ID, VOTES,
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    msg: ChallengeExecuteMsg,
) -> AppResult {
    match msg {
        ChallengeExecuteMsg::UpdateConfig {
            new_native_denom,
            new_forfeit_amount,
        } => update_config(deps, info, app, new_native_denom, new_forfeit_amount),
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
            &challenge_id,
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
        ChallengeExecuteMsg::DailyCheckIn { challenge_id } => {
            daily_check_in(deps, env, info, &app, challenge_id)
        }
        ChallengeExecuteMsg::CastVote { vote, challenge_id } => {
            cast_vote(deps, env, info, &app, vote, &challenge_id)
        }
        ChallengeExecuteMsg::CountVotes { challenge_id } => count_votes(deps, env, challenge_id),
        ChallengeExecuteMsg::ChargePenalty { challenge_id } => {
            charge_penalty(deps, challenge_id, &app)
        }
    }
}

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: ChallengeApp,
    new_native_denom: Option<String>,
    new_forfeit_amount: Option<Uint128>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let old_config = CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            native_denom: new_native_denom.unwrap_or(old_config.native_denom),
            forfeit_amount: new_forfeit_amount.unwrap_or(old_config.forfeit_amount),
        },
    )?;

    Ok(app.tag_response(Response::default(), "update_config"))
}

/// Create new Accountability
fn create_challenge(
    deps: DepsMut,
    env: Env,
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
    challenge_id: &String,
    friend_name: &String,
    friend_address: &String,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    if CHALLENGE_FRIENDS
        .may_load(deps.storage, (friend_address.clone(), challenge_id.clone()))?
        .is_some()
    {
        return Err(AppError::Std(StdError::generic_err(
            "Friend already added for this challenge",
        )));
    }

    let friend = Friend {
        address: friend_address.clone(),
        name: friend_name.clone(),
    };

    CHALLENGE_FRIENDS.save(
        deps.storage,
        (friend_address.clone(), challenge_id.clone()),
        &friend,
    )?;
    Ok(Response::new())
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

    // Ensure the friend exists for this challenge before removing
    if CHALLENGE_FRIENDS
        .may_load(deps.storage, (friend_address.clone(), challenge_id.clone()))?
        .is_none()
    {
        return Err(AppError::Std(StdError::generic_err(
            "Friend not found for this challenge",
        )));
    }

    CHALLENGE_FRIENDS.remove(deps.storage, (friend_address.clone(), challenge_id.clone()));
    Ok(Response::new())
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

    // Ensure the friends don't already exist for this challenge before adding
    for friend in friends.iter() {
        if CHALLENGE_FRIENDS
            .may_load(deps.storage, (friend.address.clone(), challenge_id.clone()))?
            .is_some()
        {
            return Err(AppError::Std(StdError::generic_err(
                "Friend already added for this challenge",
            )));
        }
    }

    // Add the friends
    for friend in friends.iter() {
        CHALLENGE_FRIENDS.save(
            deps.storage,
            (friend.address.clone(), challenge_id.clone()),
            &friend,
        )?;
    }

    Ok(Response::new())
}

fn daily_check_in(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    _challenge_id: String,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let today = date_from_block(&env);
    // Check if Admin has already checked in today
    if let Ok(check_in) = DAILY_CHECKINS.load(deps.storage, today.clone()) {
        if check_in.last_checked_in == today {
            return Err(AppError::AlreadyCheckedIn {});
        }
    }

    let _blocks_per_day = 1440; // dummy value, check this
                                // let next_interval_blocks = match FREQUENCY.load(deps.storage)? {
                                //     Frequency::EveryNBlocks(n) => n,
                                //     Frequency::Daily => blocks_per_day,
                                //     Frequency::Weekly => 7 * blocks_per_day,
                                //     Frequency::Monthly => 30 * blocks_per_day,
                                // };

    let next_check_in_block = env.block.height + 10;

    let check_in = CheckIn {
        last_checked_in: today.clone(),
        next_check_in_by: next_check_in_block,
    };

    DAILY_CHECKINS.save(deps.storage, today, &check_in)?;
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
    if let Ok(check_in) = DAILY_CHECKINS.load(deps.storage, today.clone()) {
        if check_in.last_checked_in == today {
            return Err(AppError::AlreadyCheckedIn {});
        }
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

fn count_votes(_deps: DepsMut, _env: Env, _challenge_id: String) -> AppResult {
    // Load all votes related to the given challenge_id
    Ok(Response::new().add_attribute("action", "count_votes"))
}

fn date_from_block(env: &Env) -> String {
    // Convert the block's timestamp to NaiveDateTime
    let seconds = env.block.time.seconds();
    let nano_seconds = env.block.time.subsec_nanos();
    let dt = NaiveDateTime::from_timestamp(seconds as i64, nano_seconds as u32);

    // Format the date using the NaiveDateTime object
    format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day())
}

// for now we charge the same penalty regardless of how many false votes there are,
// we may want to update this to increase the penalty amount for the number of false votes.
fn charge_penalty(deps: DepsMut, challenge_id: String, app: &ChallengeApp) -> AppResult {
    // Load the votes for the given challenge
    let votes = VOTES.load(deps.storage, challenge_id)?;

    // Check if there's any false vote
    if votes.iter().any(|vote| vote.vote == Some(false)) {
        // Fetch the penalty amount from Config
        let config: Config = CONFIG.load(deps.storage)?;
        let penalty_amount = config.forfeit_amount;

        // Deduct the penalty from the admin's balance/resource
        deduct_penalty_from_admin(deps, &penalty_amount, app)?;

        // Distribute the penalty among the friends
        //distribute_penalty_to_friends(deps, &penalty_amount, app)?;

        // Log or notify as required
        Ok(Response::new().add_attribute("action", "penalty_charged"))
    } else {
        Ok(Response::new().add_attribute("action", "no_penalty_charged"))
    }
}

fn deduct_penalty_from_admin(
    deps: DepsMut,
    penalty_amount: &Uint128,
    app: &ChallengeApp,
) -> AppResult {
    // // Fetch the admin's address from Config
    // let admin_address = deps.api.addr_validate(&config.admin)?;
    // let bank = app.bank(deps.as_ref());
    // let executor = app.executor(deps.as_ref());
    // let transfer_action = bank.transfer(
    //     vec![Asset::native(config.native_denom, *penalty_amount)],
    //     &admin_address,
    //     &executor,
    // )?;
    //
    // // Deduct the penalty from the admin's balance/resource
    // let new_balance = admin_account.balance.checked_sub(*penalty_amount)?;
    // ACCOUNTS.save(deps.storage, &admin_address, &new_account)?;
    //
    // Ok(())
    Ok(Response::new().add_attribute("action", "deduct_penalty"))
}
