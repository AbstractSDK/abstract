use abstract_app::AppError;
use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{
    wasm_execute, CosmosMsg, Decimal, DepsMut, Env, Error, MessageInfo, Order, Response, Uint128,
};
use cw_asset::{Asset, AssetList};

use crate::contract::{AppResult, ChallengeApp};

use crate::error::AppError;
use crate::msg::{AccExecuteMsg, ExecuteMsg, Frequency};
use crate::state::{
    AccEntry, ChallengeEntry, Config, Friend, CHALLENGE_FRIENDS, CHALLENGE_LIST, CONFIG, NEXT_ID,
};
use abstract_dex_adapter::api::DexInterface;
use abstract_sdk::AbstractSdkResult;
use chrono::NaiveTime;
use croncat_app::croncat_intergration_utils::{CronCatAction, CronCatTaskRequest};
use croncat_app::{CronCat, CronCatInterface};

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: ChallengeApp,
    new_native_denom: Option<String>,
    new_forfeit_amount: Option<Uint128>,
    new_refill_threshold: Option<Uint128>,
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
    source_asset: OfferAsset,
    frequence: Frequency,
    dex_name: DexName,
) -> AppResult {
    // Only the admin should be able to create a challenge.
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let config = CONFIG.load(deps.storage)?;

    // Generate the challenge id
    let id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    let dca_id = format!("acc_{id}");

    let acc_entry = AccEntry {
        source_asset,
        frequency,
    };
    CHALLENGE_LIST.save(deps.storage, dca_id.clone(), &acc_entry)?;

    let cron_cat = app.cron_cat(deps.as_ref());
    //let task_msg =

    Ok(app.tag_response(
        Response::new()
            .add_message(task_msg)
            .add_attribute("acc_id", acc_id),
        "create_accountability",
    ))
}

/// Update an existing challenge  
fn update_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    acc_id: String,
    new_source_asset: Option<OfferAsset>,
    new_frequency: Option<Frequency>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Only if frequency is changed we have to re-create a task
    let recreate_task = new_frequency.is_some();
    let old_accountability = CHALLENGE_LIST.load(deps.storage, acc_id.clone())?;
    let new_accountability = ChallengeEntry {
        name: new_name.unwrap_or(old_accountability.name),
        source_asset: new_source_asset.unwrap_or(old_accountability.source_asset),
        frequency: new_frequency.unwrap_or(old_accountability.frequency),
    };

    DCA_LIST.save(deps.storage, acc_id.clone(), &new_accountability)?;

    let response = if recreate_task {
        let config = CONFIG.load(deps.storage)?;
        let cron_cat = app.cron_cat(deps.as_ref());
        let remove_task_msg = cron_cat.remove_task(acc_id.clone())?;
        // @TODO //let create_task_msg =
    };
}

fn cancel_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: ChallengeApp,
    acc_id: String,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    CHALLENGE_LIST.remove(deps.storage, acc_id.clone());

    let cron_cat = app.cron_cat(deps.as_ref());
    let remove_task_msg = cron_cat.remove_task(acc_id.clone())?;

    Ok(app.tag_response(
        Response::new().add_message(remove_task_msg),
        "cancel_accountability",
    ))
}

fn add_friend_for_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &AccApp,
    challenge_id: u64,
    friend_name: String,
    friend_address: String,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    if CHALLENGE_FRIENDS
        .may_load(deps.storage, (friend_address, challlenge_id))?
        .is_some()
    {
        return AppError::Std(Error::generic_err(
            "Friend already added for this challenge",
        ));
    }

    let friend = Friend {
        address: friend_address.clone(),
        name: friend_name,
    };

    CHALLENGE_FRIENDS.save(deps.storage, (friend_address, challlenge_id), &friend)?;
    Ok(Response::new())
}

pub fn remove_friend_from_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    friend_address: String,
) -> AppResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Ensure the friend exists for this challenge before removing
    if CHALLENGE_FRIENDS
        .may_load(deps.storage, (friend_address, challenge_id))?
        .is_none()
    {
        return Err(AppError::Std(Error::generic_err(
            "Friend not found for this challenge",
        )));
    }

    CHALLENGE_FRIENDS.remove(deps.storage, (friend_address, challenge_id));
    Ok(Response::new())
}

fn add_friends_for_challenge(
    deps: DepsMut,
    info: MessageInfo,
    app: &ChallengeApp,
    challenge_id: u64,
    friends: Vec<Friend>,
) -> AppResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Ensure the friends don't already exist for this challenge before adding
    for friend in friends.iter() {
        if CHALLENGE_FRIENDS
            .may_load(deps.storage, (friend.address, challenge_id))?
            .is_some()
        {
            return Err(AppError::Std(Error::generic_err(
                "Friend already added for this challenge",
            )));
        }
    }

    // Add the friends
    for friend in friends.iter() {
        CHALLENGE_FRIENDS.save(deps.storage, (friend.address, challenge_id), &friend)?;
    }

    Ok(Response::new())
}

fn daily_check_in(deps: DepsMut, env: Env, info: MessageInfo, app: &ChallengeApp) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Check if Admin has already checked in today
    if let Ok(check_in) = DAILY_CHECKINS.load(deps.storage, &date_from_block(env)) {
        if check_in.last_checked_in == today {
            return Err(StdError::generic_err("Already checked in today."));
        }
    }

    let blocks_per_day = 1440; // dummy value, check this
    let next_interval_blocks = match FREQUENCY.load(deps.storage)? {
        Frequency::EveryNBlocks(n) => n,
        Frequency::Daily => blocks_per_day,
        Frequency::Weekly => 7 * blocks_per_day,
        Frequency::Monthly => 30 * blocks_per_day,
    };

    let next_check_in_block = env.block.height + next_interval_blocks;

    let check_in = CheckIn {
        last_checked_in: today,
        next_check_in_by: next_check_in_block,
    };

    DAILY_CHECKINS.save(deps.storage, &today, &check_in)?;
    Ok(Response::new().add_attribute("action", "check_in"))
}

fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: &ChallengeApp,
    vote: Option<bool>,
) -> AppResult {
    // Ensure sender is a registered friend
    if !FRIENDS.has(deps.storage, &info.sender.to_string()) {
        return Err(StdError::generic_err("Only registered friends can vote."));
    }

    // If Admin checked in today, friends can't vote
    if let Ok(check_in) = DAILY_CHECKINS.load(deps.storage, &date_from_block(env)) {
        if check_in.last_checked_in == today {
            return Err(StdError::generic_err(
                "Joe checked in today, no need to vote.",
            ));
        }
    }

    // If the vote is None, default to true (meaning we assume Admin fulfilled his challenge)
    let final_vote = vote.unwrap_or(true);

    // Save the vote
    let vote_entry = Vote {
        voter: info.sender.to_string(),
        vote: Some(final_vote),
        challenge_id: app.current_challenge_id,
    };
    VOTES.save(deps.storage, &env.block.height, &vote_entry)?;
    Ok(Response::new().add_attribute("action", "cast_vote"))
}

fn count_votes(deps: DepsMut, env: Env, challenge_id: u64) -> Result<Response, StdError> {
    // Load all votes related to the given challenge_id
    let votes_for_challenge: Vec<Vote> = VOTES
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|(_, vote)| {
            if vote.challenge_id == challenge_id {
                Some(vote)
            } else {
                None
            }
        })
        .collect();

    if votes_for_challenge.is_empty() {
        return Err(StdError::generic_err("No votes found for this challenge."));
    }

    let total_votes = votes_for_challenge.len() as u64;
    let passed_votes = votes_for_challenge
        .iter()
        .filter(|vote| vote.vote.unwrap_or(true))
        .count() as u64;
    let failed_votes = total_votes - passed_votes;

    let result = if passed_votes > failed_votes {
        "passed"
    } else {
        "failed"
    };

    Ok(Response::new()
        .add_attribute("action", "count_votes")
        .add_attribute("challenge_id", challenge_id.to_string())
        .add_attribute("total_votes", total_votes.to_string())
        .add_attribute("passed_votes", passed_votes.to_string())
        .add_attribute("failed_votes", failed_votes.to_string())
        .add_attribute("result", result))
}

fn date_from_block(env: Env) -> String {
    // Convert the block's timestamp to NaiveDateTime
    let seconds = env.block.time.seconds();
    let nano_seconds = env.block.time.subsec_nanos();
    let dt = NaiveDateTime::from_timestamp(seconds as i64, nano_seconds as u32);

    // Format the date using the NaiveDateTime object
    format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day())
}
