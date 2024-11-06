use abstract_app::sdk::{
    features::{AbstractNameService, AbstractResponse},
    TransferInterface,
};
use abstract_app::std::objects::AssetEntry;
use chrono::{DateTime, FixedOffset, LocalResult, NaiveTime, TimeZone};
use cosmwasm_std::{
    BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, Int64, MessageInfo, StdError, Uint128,
};
use cw_asset::AssetInfoBase;
use cw_utils::must_pay;

use crate::{
    contract::{CalendarApp, CalendarAppResult},
    error::CalendarError,
    msg::CalendarExecuteMsg,
    state::{Meeting, CALENDAR, CONFIG},
};

enum StakeAction {
    Return,
    FullSlash,
    PartialSlash { minutes_late: u32 },
}

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: CalendarApp,
    msg: CalendarExecuteMsg,
) -> CalendarAppResult {
    match msg {
        CalendarExecuteMsg::RequestMeeting {
            start_time,
            end_time,
        } => request_meeting(deps, info, module, env, start_time, end_time),
        CalendarExecuteMsg::SlashFullStake {
            day_datetime,
            meeting_index,
        } => handle_stake(
            deps,
            info,
            module,
            env,
            day_datetime,
            meeting_index,
            StakeAction::FullSlash,
        ),
        CalendarExecuteMsg::SlashPartialStake {
            day_datetime,
            meeting_index,
            minutes_late,
        } => handle_stake(
            deps,
            info,
            module,
            env,
            day_datetime,
            meeting_index,
            StakeAction::PartialSlash { minutes_late },
        ),
        CalendarExecuteMsg::ReturnStake {
            day_datetime,
            meeting_index,
        } => handle_stake(
            deps,
            info,
            module,
            env,
            day_datetime,
            meeting_index,
            StakeAction::Return,
        ),
        CalendarExecuteMsg::UpdateConfig {
            price_per_minute,
            denom,
        } => update_config(deps, env, info, module, price_per_minute, denom),
    }
}

fn request_meeting(
    deps: DepsMut,
    info: MessageInfo,
    module: CalendarApp,
    env: Env,
    meeting_start_time: Int64,
    meeting_end_time: Int64,
) -> CalendarAppResult {
    let config = CONFIG.load(deps.storage)?;
    let amount_sent = must_pay(&info, &config.denom)?;

    let timezone: FixedOffset =
        FixedOffset::east_opt(config.utc_offset).ok_or(CalendarError::InvalidUtcOffset {})?;

    let meeting_start_datetime: DateTime<FixedOffset> =
        get_date_time(timezone, meeting_start_time)?;
    let meeting_start_time: NaiveTime = meeting_start_datetime.time();

    let meeting_end_datetime = get_date_time(timezone, meeting_end_time)?;
    let meeting_end_time: NaiveTime = meeting_end_datetime.time();

    let meeting: Meeting = Meeting::new(
        &config,
        &env.block,
        meeting_start_datetime,
        meeting_end_datetime,
        info.sender,
        amount_sent,
    )?;

    let meeting_start_timestamp = meeting.start_time;
    let meeting_end_timestamp = meeting.end_time;

    // This number will be positive enforced by previous checks.
    let duration_in_minutes: Uint128 =
        Uint128::new((meeting_end_time - meeting_start_time).num_minutes() as u128);

    let expected_amount = duration_in_minutes * config.price_per_minute;
    if amount_sent != expected_amount {
        return Err(CalendarError::InvalidStakeAmountSent { expected_amount });
    }

    // Get unix start date of the current day
    let start_of_day_timestamp: i64 = meeting_start_datetime
        .date_naive()
        .and_time(NaiveTime::default())
        .and_utc()
        .timestamp();

    let mut existing_meetings: Vec<Meeting> = CALENDAR
        .may_load(deps.storage, start_of_day_timestamp)?
        .unwrap_or_default();

    if !existing_meetings.is_empty() {
        //Validate that there are no colisions.
        for meeting in existing_meetings.iter() {
            let start_time_conflicts = meeting.start_time <= meeting_start_timestamp
                && meeting_start_timestamp < meeting.end_time;

            let end_time_conflicts = meeting.start_time < meeting_end_timestamp
                && meeting_end_timestamp <= meeting.end_time;

            if start_time_conflicts || end_time_conflicts {
                return Err(CalendarError::MeetingConflictExists {});
            }
        }
    }
    existing_meetings.push(meeting);

    CALENDAR.save(deps.storage, start_of_day_timestamp, &existing_meetings)?;

    Ok(module
        .response("request_meeting")
        .add_attribute("meeting_start_time", meeting_start_timestamp.to_string())
        .add_attribute("meeting_end_time", meeting_end_timestamp.to_string()))
}

fn handle_stake(
    deps: DepsMut,
    info: MessageInfo,
    module: CalendarApp,
    env: Env,
    day_datetime: Int64,
    meeting_index: u32,
    stake_action: StakeAction,
) -> CalendarAppResult {
    module
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;

    let config = CONFIG.load(deps.storage)?;

    let meetings = CALENDAR.may_load(deps.storage, day_datetime.i64())?;
    if meetings.is_none() {
        return Err(CalendarError::NoMeetingsAtGivenDayDateTime {});
    }
    let mut meetings = meetings.unwrap();
    if meeting_index as usize >= meetings.len() {
        return Err(CalendarError::MeetingDoesNotExist {});
    }
    let meeting: &mut Meeting = meetings.get_mut(meeting_index as usize).unwrap();

    if (env.block.time.seconds() as i64) <= meeting.end_time {
        return Err(CalendarError::MeetingNotFinishedYet {});
    }

    let amount_staked = meeting.amount_staked;
    let requester = meeting.requester.to_string();
    if amount_staked.is_zero() {
        return Err(CalendarError::StakeAlreadyHandled {});
    }

    meeting.amount_staked = Uint128::zero();
    let bank = module.bank(deps.as_ref());

    let response = match stake_action {
        StakeAction::Return => module.response("return_stake").add_message(BankMsg::Send {
            to_address: requester,
            amount: vec![Coin::new(amount_staked, config.denom)],
        }),
        StakeAction::FullSlash => {
            let account_deposit_msgs: Vec<CosmosMsg> =
                bank.deposit(vec![Coin::new(amount_staked, config.denom)])?;
            module
                .response("full_slash")
                .add_messages(account_deposit_msgs)
        }
        StakeAction::PartialSlash { minutes_late } => {
            // Cast should be safe given we cannot have a meeting longer than 24 hours.
            let meeting_duration_in_minutes: u32 =
                ((meeting.end_time - meeting.start_time) / 60) as u32;
            if minutes_late > meeting_duration_in_minutes {
                return Err(CalendarError::MinutesLateCannotExceedDurationOfMeeting {});
            }
            let amount_to_slash =
                amount_staked.multiply_ratio(minutes_late, meeting_duration_in_minutes as u128);

            let account_deposit_msgs: Vec<CosmosMsg> =
                bank.deposit(vec![Coin::new(amount_to_slash, config.denom.clone())])?;

            module
                .response("partial_slash")
                .add_message(BankMsg::Send {
                    to_address: requester,
                    amount: vec![Coin::new(amount_staked - amount_to_slash, config.denom)],
                })
                .add_messages(account_deposit_msgs)
        }
    };

    CALENDAR.save(deps.storage, day_datetime.i64(), &meetings)?;

    Ok(response)
}

fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: CalendarApp,
    price_per_minute: Option<Uint128>,
    denom: Option<AssetEntry>,
) -> CalendarAppResult {
    module
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut attrs = vec![];
    if let Some(price_per_minute) = price_per_minute {
        config.price_per_minute = price_per_minute;
        attrs.push(("price_per_minute", price_per_minute.to_string()));
    }
    if let Some(unresolved) = denom {
        let denom = resolve_native_ans_denom(deps.as_ref(), &env, &module, unresolved.clone())?;
        config.denom = denom;
        attrs.push(("denom", unresolved.to_string()));
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(module.custom_response("update_config", attrs))
}

pub fn resolve_native_ans_denom(
    deps: Deps,
    env: &Env,
    module: &CalendarApp,
    denom: AssetEntry,
) -> CalendarAppResult<String> {
    let name_service = module.name_service(deps);
    let resolved_denom = name_service.query(&denom)?;
    let denom = match resolved_denom {
        AssetInfoBase::Native(denom) => Ok(denom),
        _ => Err(StdError::generic_err("Non-native denom not supported")),
    }?;
    Ok(denom)
}

fn get_date_time(
    timezone: FixedOffset,
    timestamp: Int64,
) -> CalendarAppResult<DateTime<FixedOffset>> {
    if let LocalResult::Single(value) = timezone.timestamp_opt(timestamp.i64(), 0) {
        Ok(value)
    } else {
        Err(CalendarError::InvalidTime {})
    }
}
