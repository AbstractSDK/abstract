use abstract_client::{application::Application, client::AbstractClient, publisher::Publisher};
use abstract_core::objects::{gov_type::GovernanceDetails, AssetEntry};
use calendar_app::{
    error::CalendarError,
    msg::{CalendarExecuteMsg, CalendarInstantiateMsg, ConfigResponse, Time},
    state::Meeting,
    *,
};
use chrono::{DateTime, Days, FixedOffset, NaiveDateTime, NaiveTime, TimeZone, Timelike};
use cw_asset::AssetInfoUnchecked;
// Use prelude to get all the necessary imports
use abstract_testing::prelude::*;
use cw_orch::{anyhow, prelude::*};

use cosmwasm_std::{coins, Addr, BlockInfo, Uint128};

// consts for testing
const DENOM: &str = "juno>stake";

const INITIAL_BALANCE: u128 = 10_000;

fn request_meeting_with_start_time(
    day_datetime: DateTime<FixedOffset>,
    start_time: Time,
    app: CalendarAppInterface<Mock>,
) -> anyhow::Result<(NaiveDateTime, NaiveDateTime)> {
    request_meeting(
        day_datetime,
        start_time,
        Time {
            hour: start_time.hour + 1,
            minute: start_time.minute,
        },
        app,
        Coin::new(60, DENOM),
    )
}

fn request_meeting_with_end_time(
    day_datetime: DateTime<FixedOffset>,
    end_time: Time,
    app: CalendarAppInterface<Mock>,
) -> anyhow::Result<(NaiveDateTime, NaiveDateTime)> {
    request_meeting(
        day_datetime,
        Time {
            hour: end_time.hour - 1,
            minute: end_time.minute,
        },
        end_time,
        app,
        Coin::new(60, DENOM),
    )
}

fn request_meeting(
    day_datetime: DateTime<FixedOffset>,
    start_time: Time,
    end_time: Time,
    app: CalendarAppInterface<Mock>,
    funds: Coin,
) -> anyhow::Result<(NaiveDateTime, NaiveDateTime)> {
    let meeting_start_datetime: NaiveDateTime = day_datetime
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(start_time.hour, start_time.minute, 0).unwrap());

    let meeting_end_datetime: NaiveDateTime = meeting_start_datetime
        .with_hour(end_time.hour)
        .unwrap()
        .with_minute(end_time.minute)
        .unwrap();

    app.request_meeting(
        meeting_end_datetime.timestamp().into(),
        meeting_start_datetime.timestamp().into(),
        &[funds],
    )?;

    Ok((meeting_start_datetime, meeting_end_datetime))
}

#[allow(clippy::type_complexity)]
fn setup_with_time(
    start_time: Time,
    end_time: Time,
) -> anyhow::Result<(
    Application<Mock, CalendarAppInterface<Mock>>,
    AbstractClient<Mock>,
)> {
    let client: AbstractClient<Mock> = AbstractClient::builder(OWNER.to_owned())
        .balance("sender1", coins(INITIAL_BALANCE, DENOM))
        .balance("sender2", coins(INITIAL_BALANCE, DENOM))
        .balance("sender", coins(INITIAL_BALANCE, DENOM))
        .asset(DENOM, AssetInfoUnchecked::native(DENOM))
        .build()?;

    // Create account to install app onto as well as claim namespace.
    let publisher: Publisher<Mock> = client
        .publisher_builder()
        .governance_details(GovernanceDetails::Monarchy {
            monarch: OWNER.to_owned(),
        })
        .namespace("my-namespace")
        .build()?;

    publisher.publish_app::<CalendarAppInterface<Mock>>()?;

    let app: Application<Mock, CalendarAppInterface<Mock>> = publisher.install_app(
        &CalendarInstantiateMsg {
            price_per_minute: Uint128::from(1u128),
            denom: AssetEntry::from(DENOM),
            utc_offset: 0,
            start_time,
            end_time,
        },
        &[],
    )?;

    Ok((app, client))
}

/// Set up the test environment with the contract installed
#[allow(clippy::type_complexity)]
fn setup() -> anyhow::Result<(
    Application<Mock, CalendarAppInterface<Mock>>,
    AbstractClient<Mock>,
)> {
    setup_with_time(
        Time { hour: 9, minute: 0 },
        Time {
            hour: 17,
            minute: 0,
        },
    )
}

#[test]
fn start_hour_out_of_bounds() -> anyhow::Result<()> {
    // Cannot call `.unwrap_err` since AbstractAccount does not implement `Debug`.
    // https://stackoverflow.com/questions/75088004/unwrap-err-function-seems-to-be-returning-t-rather-than-e
    if let Err(error) = setup_with_time(
        Time {
            hour: 24,
            minute: 0,
        },
        Time {
            hour: 10,
            minute: 0,
        },
    ) {
        assert_eq!(
            CalendarError::HourOutOfBounds {}.to_string(),
            error.root_cause().to_string()
        );
        return Ok(());
    }
    panic!("Expected error");
}

#[test]
fn end_hour_out_of_bounds() -> anyhow::Result<()> {
    // Cannot call `.unwrap_err` since AbstractAccount does not implement `Debug`.
    // https://stackoverflow.com/questions/75088004/unwrap-err-function-seems-to-be-returning-t-rather-than-e
    if let Err(error) = setup_with_time(
        Time {
            hour: 10,
            minute: 0,
        },
        Time {
            hour: 24,
            minute: 0,
        },
    ) {
        assert_eq!(
            CalendarError::HourOutOfBounds {}.to_string(),
            error.root_cause().to_string()
        );
        return Ok(());
    }
    panic!("Expected error");
}

#[test]
fn start_minutes_out_of_bounds() -> anyhow::Result<()> {
    // Cannot call `.unwrap_err` since AbstractAccount does not implement `Debug`.
    // https://stackoverflow.com/questions/75088004/unwrap-err-function-seems-to-be-returning-t-rather-than-e
    if let Err(error) = setup_with_time(
        Time {
            hour: 10,
            minute: 60,
        },
        Time {
            hour: 13,
            minute: 0,
        },
    ) {
        assert_eq!(
            CalendarError::MinutesOutOfBounds {}.to_string(),
            error.root_cause().to_string()
        );
        return Ok(());
    }
    panic!("Expected error");
}

#[test]
fn end_minutes_out_of_bounds() -> anyhow::Result<()> {
    // Cannot call `.unwrap_err` since AbstractAccount does not implement `Debug`.
    // https://stackoverflow.com/questions/75088004/unwrap-err-function-seems-to-be-returning-t-rather-than-e
    if let Err(error) = setup_with_time(
        Time {
            hour: 10,
            minute: 0,
        },
        Time {
            hour: 13,
            minute: 60,
        },
    ) {
        assert_eq!(
            CalendarError::MinutesOutOfBounds {}.to_string(),
            error.root_cause().to_string()
        );
        return Ok(());
    }
    panic!("Expected error");
}

#[test]
fn start_time_after_end_time() -> anyhow::Result<()> {
    // Cannot call `.unwrap_err` since AbstractAccount does not implement `Debug`.
    // https://stackoverflow.com/questions/75088004/unwrap-err-function-seems-to-be-returning-t-rather-than-e
    if let Err(error) = setup_with_time(
        Time {
            hour: 13,
            minute: 0,
        },
        Time {
            hour: 10,
            minute: 0,
        },
    ) {
        assert_eq!(
            CalendarError::EndTimeMustBeAfterStartTime {}.to_string(),
            error.root_cause().to_string()
        );
        return Ok(());
    }
    panic!("Expected error");
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (app, _client) = setup()?;

    let config = app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            price_per_minute: Uint128::from(1u128),
            utc_offset: 0,
            start_time: Time { hour: 9, minute: 0 },
            end_time: Time {
                hour: 17,
                minute: 0,
            },
        }
    );
    Ok(())
}

#[test]
fn request_meeting_at_start_of_day() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let (meeting_start_datetime, meeting_end_datetime) = request_meeting_with_start_time(
        current_datetime.checked_add_days(Days::new(1)).unwrap(),
        config.start_time,
        app.clone(),
    )?;

    let meetings_response = app.meetings(
        meeting_start_datetime
            .date()
            .and_time(NaiveTime::default())
            .timestamp()
            .into(),
    )?;

    assert_eq!(
        vec![Meeting {
            start_time: meeting_start_datetime.timestamp(),
            end_time: meeting_end_datetime.timestamp(),
            requester: sender,
            amount_staked: Uint128::from(60u128),
        }],
        meetings_response.meetings
    );

    Ok(())
}

#[test]
fn request_meeting_at_end_of_day() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let (meeting_start_datetime, meeting_end_datetime) = request_meeting_with_end_time(
        current_datetime.checked_add_days(Days::new(1)).unwrap(),
        config.end_time,
        app.clone(),
    )?;

    let meetings_response = app.meetings(
        meeting_start_datetime
            .date()
            .and_time(NaiveTime::default())
            .timestamp()
            .into(),
    )?;

    assert_eq!(
        vec![Meeting {
            start_time: meeting_start_datetime.timestamp(),
            end_time: meeting_end_datetime.timestamp(),
            requester: sender,
            amount_staked: Uint128::from(60u128),
        }],
        meetings_response.meetings
    );

    Ok(())
}

#[test]
fn request_multiple_meetings_on_same_day() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender1 = Addr::unchecked("sender1");
    app.set_sender(&sender1);

    let (meeting_start_datetime1, meeting_end_datetime1) = request_meeting_with_start_time(
        day_datetime,
        Time {
            hour: 11,
            minute: 30,
        },
        app.clone(),
    )?;

    let sender2 = Addr::unchecked("sender2");
    app.set_sender(&sender2);

    let (meeting_start_datetime2, meeting_end_datetime2) = request_meeting_with_start_time(
        day_datetime,
        Time {
            hour: 13,
            minute: 0,
        },
        app.clone(),
    )?;
    let meetings_response = app.meetings(
        meeting_start_datetime1
            .date()
            .and_time(NaiveTime::default())
            .timestamp()
            .into(),
    )?;

    assert_eq!(
        vec![
            Meeting {
                start_time: meeting_start_datetime1.timestamp(),
                end_time: meeting_end_datetime1.timestamp(),
                requester: sender1,
                amount_staked: Uint128::from(60u128),
            },
            Meeting {
                start_time: meeting_start_datetime2.timestamp(),
                end_time: meeting_end_datetime2.timestamp(),
                requester: sender2,
                amount_staked: Uint128::from(60u128),
            }
        ],
        meetings_response.meetings
    );

    Ok(())
}

#[test]
fn request_back_to_back_meetings_on_left() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender1 = Addr::unchecked("sender1");
    app.set_sender(&sender1);

    let (meeting_start_datetime1, meeting_end_datetime1) = request_meeting(
        day_datetime,
        Time {
            hour: 11,
            minute: 30,
        },
        Time {
            hour: 12,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )?;

    let sender2 = Addr::unchecked("sender2");
    app.set_sender(&sender2);

    let (meeting_start_datetime2, meeting_end_datetime2) = request_meeting(
        day_datetime,
        Time {
            hour: 10,
            minute: 30,
        },
        Time {
            hour: 11,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )?;
    let meetings_response = app.meetings(
        meeting_start_datetime1
            .date()
            .and_time(NaiveTime::default())
            .timestamp()
            .into(),
    )?;

    assert_eq!(
        vec![
            Meeting {
                start_time: meeting_start_datetime1.timestamp(),
                end_time: meeting_end_datetime1.timestamp(),
                requester: sender1,
                amount_staked: Uint128::from(60u128),
            },
            Meeting {
                start_time: meeting_start_datetime2.timestamp(),
                end_time: meeting_end_datetime2.timestamp(),
                requester: sender2,
                amount_staked: Uint128::from(60u128),
            }
        ],
        meetings_response.meetings
    );

    Ok(())
}

#[test]
fn request_back_to_back_meetings_on_right() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender1 = Addr::unchecked("sender1");
    app.set_sender(&sender1);

    let (meeting_start_datetime1, meeting_end_datetime1) = request_meeting(
        day_datetime,
        Time {
            hour: 11,
            minute: 30,
        },
        Time {
            hour: 12,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )?;

    let sender2 = Addr::unchecked("sender2");
    app.set_sender(&sender2);

    let (meeting_start_datetime2, meeting_end_datetime2) = request_meeting(
        day_datetime,
        Time {
            hour: 12,
            minute: 30,
        },
        Time {
            hour: 13,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )?;
    let meetings_response = app.meetings(
        meeting_start_datetime1
            .date()
            .and_time(NaiveTime::default())
            .timestamp()
            .into(),
    )?;

    assert_eq!(
        vec![
            Meeting {
                start_time: meeting_start_datetime1.timestamp(),
                end_time: meeting_end_datetime1.timestamp(),
                requester: sender1,
                amount_staked: Uint128::from(60u128),
            },
            Meeting {
                start_time: meeting_start_datetime2.timestamp(),
                end_time: meeting_end_datetime2.timestamp(),
                requester: sender2,
                amount_staked: Uint128::from(60u128),
            }
        ],
        meetings_response.meetings
    );

    Ok(())
}

#[test]
fn request_meetings_on_different_days() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let sender1 = Addr::unchecked("sender1");
    app.set_sender(&sender1);

    let (meeting_start_datetime1, meeting_end_datetime1) = request_meeting_with_start_time(
        current_datetime.checked_add_days(Days::new(1)).unwrap(),
        Time {
            hour: 11,
            minute: 30,
        },
        app.clone(),
    )?;

    let sender2 = Addr::unchecked("sender2");
    app.set_sender(&sender2);

    let (meeting_start_datetime2, meeting_end_datetime2) = request_meeting_with_start_time(
        current_datetime.checked_add_days(Days::new(2)).unwrap(),
        Time {
            hour: 11,
            minute: 30,
        },
        app.clone(),
    )?;
    let meetings_response1 = app.meetings(
        meeting_start_datetime1
            .date()
            .and_time(NaiveTime::default())
            .timestamp()
            .into(),
    )?;

    assert_eq!(
        vec![Meeting {
            start_time: meeting_start_datetime1.timestamp(),
            end_time: meeting_end_datetime1.timestamp(),
            requester: sender1,
            amount_staked: Uint128::from(60u128),
        }],
        meetings_response1.meetings
    );

    let meetings_response2 = app.meetings(
        meeting_start_datetime2
            .date()
            .and_time(NaiveTime::default())
            .timestamp()
            .into(),
    )?;

    assert_eq!(
        vec![Meeting {
            start_time: meeting_start_datetime2.timestamp(),
            end_time: meeting_end_datetime2.timestamp(),
            requester: sender2,
            amount_staked: Uint128::from(60u128),
        }],
        meetings_response2.meetings
    );

    Ok(())
}

#[test]
fn cannot_request_multiple_meetings_with_same_start_time() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender1 = Addr::unchecked("sender1");
    app.set_sender(&sender1);

    request_meeting_with_start_time(
        day_datetime,
        Time {
            hour: 11,
            minute: 30,
        },
        app.clone(),
    )?;

    let sender2 = Addr::unchecked("sender2");
    app.set_sender(&sender2);

    let error = request_meeting_with_start_time(
        day_datetime,
        Time {
            hour: 11,
            minute: 30,
        },
        app.clone(),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::MeetingConflictExists {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_contained_in_another() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender1 = Addr::unchecked("sender1");
    app.set_sender(&sender1);

    request_meeting(
        day_datetime,
        Time {
            hour: 11,
            minute: 0,
        },
        Time {
            hour: 13,
            minute: 0,
        },
        app.clone(),
        Coin::new(120, DENOM),
    )?;

    let sender2 = Addr::unchecked("sender2");
    app.set_sender(&sender2);

    let error = request_meeting(
        day_datetime,
        Time {
            hour: 12,
            minute: 0,
        },
        Time {
            hour: 12,
            minute: 30,
        },
        app.clone(),
        Coin::new(30, DENOM),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::MeetingConflictExists {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_left_intersection() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender1 = Addr::unchecked("sender1");
    app.set_sender(&sender1);

    request_meeting(
        day_datetime,
        Time {
            hour: 11,
            minute: 0,
        },
        Time {
            hour: 13,
            minute: 0,
        },
        app.clone(),
        Coin::new(120, DENOM),
    )?;

    let sender2 = Addr::unchecked("sender2");
    app.set_sender(&sender2);

    let error = request_meeting(
        day_datetime,
        Time {
            hour: 10,
            minute: 30,
        },
        Time {
            hour: 11,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::MeetingConflictExists {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_right_intersection() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender1 = Addr::unchecked("sender1");
    app.set_sender(&sender1);

    request_meeting(
        day_datetime,
        Time {
            hour: 11,
            minute: 0,
        },
        Time {
            hour: 13,
            minute: 0,
        },
        app.clone(),
        Coin::new(120, DENOM),
    )?;

    let sender2 = Addr::unchecked("sender2");
    app.set_sender(&sender2);

    let error = request_meeting(
        day_datetime,
        Time {
            hour: 12,
            minute: 30,
        },
        Time {
            hour: 13,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::MeetingConflictExists {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_in_past() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_sub_days(Days::new(1)).unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let error = request_meeting(
        day_datetime,
        Time {
            hour: 12,
            minute: 30,
        },
        Time {
            hour: 13,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::StartTimeMustBeInFuture {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_end_time_before_start_time() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let error = request_meeting(
        day_datetime,
        Time {
            hour: 13,
            minute: 30,
        },
        Time {
            hour: 12,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::EndTimeMustBeAfterStartTime {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_start_time_out_of_calendar_bounds() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let error = request_meeting(
        day_datetime,
        Time {
            hour: config.start_time.hour - 1,
            minute: 30,
        },
        Time {
            hour: 12,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::OutOfBoundsStartTime {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_end_time_out_of_calendar_bounds() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let day_datetime = current_datetime.checked_add_days(Days::new(1)).unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let error = request_meeting(
        day_datetime,
        Time {
            hour: 12,
            minute: 30,
        },
        Time {
            hour: config.end_time.hour + 1,
            minute: 30,
        },
        app.clone(),
        Coin::new(60, DENOM),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::OutOfBoundsEndTime {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_start_and_end_being_on_different_days() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let meeting_start_datetime: NaiveDateTime = current_datetime
        .checked_add_days(Days::new(1))
        .unwrap()
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap());

    let meeting_end_datetime: NaiveDateTime = current_datetime
        .checked_add_days(Days::new(2))
        .unwrap()
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());

    let error: anyhow::Error = app
        .execute(
            &abstract_core::base::ExecuteMsg::Module(CalendarExecuteMsg::RequestMeeting {
                start_time: meeting_start_datetime.timestamp().into(),
                end_time: meeting_end_datetime.timestamp().into(),
            }),
            Some(&[Coin::new(60, DENOM)]),
        )
        .unwrap_err()
        .into();

    assert_eq!(
        CalendarError::StartAndEndTimeNotOnSameDay {}.to_string(),
        error.root_cause().to_string(),
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_insufficient_funds() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let error: anyhow::Error = request_meeting(
        current_datetime.checked_add_days(Days::new(1)).unwrap(),
        Time {
            hour: 10,
            minute: 0,
        },
        Time {
            hour: 11,
            minute: 0,
        },
        app.clone(),
        Coin::new(30, DENOM),
    )
    .unwrap_err();

    assert_eq!(
        CalendarError::InvalidStakeAmountSent {
            expected_amount: Uint128::from(60u128)
        }
        .to_string(),
        error.root_cause().to_string(),
    );

    Ok(())
}

#[test]
fn slash_full_stake() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;
    let admin = app.account().owner()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let (meeting_start_datetime, meeting_end_datetime) = request_meeting_with_start_time(
        current_datetime.checked_add_days(Days::new(1)).unwrap(),
        config.start_time,
        app.clone(),
    )?;

    client.wait_blocks(100000)?;

    let day_datetime = meeting_start_datetime
        .date()
        .and_time(NaiveTime::default())
        .timestamp();

    app.set_sender(&admin);
    app.slash_full_stake(day_datetime.into(), 0)?;

    let meetings_response = app.meetings(day_datetime.into())?;

    assert_eq!(
        vec![Meeting {
            start_time: meeting_start_datetime.timestamp(),
            end_time: meeting_end_datetime.timestamp(),
            requester: sender,
            amount_staked: Uint128::zero(),
        }],
        meetings_response.meetings
    );

    assert_eq!(Uint128::from(60u128), app.account().query_balance(DENOM)?);

    Ok(())
}

#[test]
fn return_stake() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;
    let admin = app.account().owner()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let (meeting_start_datetime, meeting_end_datetime) = request_meeting_with_start_time(
        current_datetime.checked_add_days(Days::new(1)).unwrap(),
        config.start_time,
        app.clone(),
    )?;

    assert_eq!(
        Uint128::from(INITIAL_BALANCE - 60),
        client.query_balance(&sender, DENOM)?
    );

    client.wait_blocks(100000)?;

    let day_datetime = meeting_start_datetime
        .date()
        .and_time(NaiveTime::default())
        .timestamp();

    app.set_sender(&admin);
    app.return_stake(day_datetime.into(), 0)?;

    let meetings_response = app.meetings(day_datetime.into())?;

    assert_eq!(
        vec![Meeting {
            start_time: meeting_start_datetime.timestamp(),
            end_time: meeting_end_datetime.timestamp(),
            requester: sender.clone(),
            amount_staked: Uint128::zero(),
        }],
        meetings_response.meetings
    );

    assert_eq!(
        Uint128::from(INITIAL_BALANCE),
        client.query_balance(&sender, DENOM)?
    );

    Ok(())
}

#[test]
fn slash_partial_stake() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (mut app, client) = setup()?;
    let block_info: BlockInfo = client.block_info()?;
    let admin = app.account().owner()?;

    let config: ConfigResponse = app.config()?;

    let timezone: FixedOffset = FixedOffset::east_opt(config.utc_offset).unwrap();
    let current_datetime = timezone
        .timestamp_opt(block_info.time.seconds() as i64, 0)
        .unwrap();

    let sender = Addr::unchecked("sender");
    app.set_sender(&sender);

    let (meeting_start_datetime, meeting_end_datetime) = request_meeting_with_start_time(
        current_datetime.checked_add_days(Days::new(1)).unwrap(),
        config.start_time,
        app.clone(),
    )?;

    assert_eq!(
        Uint128::from(INITIAL_BALANCE - 60),
        client.query_balance(&sender, DENOM)?
    );

    client.wait_blocks(100000)?;

    let day_datetime = meeting_start_datetime
        .date()
        .and_time(NaiveTime::default())
        .timestamp();

    app.set_sender(&admin);
    // 20 minutes late for a 60 minute meeting
    app.slash_partial_stake(day_datetime.into(), 0, 20)?;

    let meetings_response = app.meetings(day_datetime.into())?;

    assert_eq!(
        vec![Meeting {
            start_time: meeting_start_datetime.timestamp(),
            end_time: meeting_end_datetime.timestamp(),
            requester: sender.clone(),
            amount_staked: Uint128::zero(),
        }],
        meetings_response.meetings
    );

    assert_eq!(
        Uint128::from(INITIAL_BALANCE - 20),
        client.query_balance(&sender, DENOM)?
    );

    assert_eq!(Uint128::from(20u128), app.account().query_balance(DENOM)?);

    Ok(())
}
