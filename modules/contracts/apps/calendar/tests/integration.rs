use abstract_core::objects::{gov_type::GovernanceDetails, AccountId, AssetEntry};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, DeployStrategy, VCExecFns};
use calendar_app::{
    contract::{APP_ID, APP_VERSION},
    error::AppError,
    msg::{AppExecuteMsg, AppInstantiateMsg, ConfigResponse, Time},
    state::Meeting,
    *,
};
use chrono::{DateTime, Days, FixedOffset, NaiveDateTime, NaiveTime, TimeZone, Timelike};
use cw_asset::AssetInfo;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{coins, Addr, BlockInfo, Uint128};

// consts for testing
const ADMIN: &str = "admin";
const DENOM: &str = "juno>stake";

const INITIAL_BALANCE: u128 = 10_000;

fn request_meeting_with_start_time(
    day_datetime: DateTime<FixedOffset>,
    start_time: Time,
    app: AppInterface<Mock>,
) -> anyhow::Result<(NaiveDateTime, NaiveDateTime)> {
    request_meeting(
        day_datetime,
        start_time.clone(),
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
    app: AppInterface<Mock>,
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
    app: AppInterface<Mock>,
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

/// Set up the test environment with the contract installed
#[allow(clippy::type_complexity)]
fn setup() -> anyhow::Result<(
    AbstractAccount<Mock>,
    Abstract<Mock>,
    AppInterface<Mock>,
    Mock,
)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // set balances
    mock.set_balance(&Addr::unchecked("sender1"), coins(INITIAL_BALANCE, DENOM))?;
    mock.set_balance(&Addr::unchecked("sender2"), coins(INITIAL_BALANCE, DENOM))?;
    mock.set_balance(&Addr::unchecked("sender"), coins(INITIAL_BALANCE, DENOM))?;

    // Construct the contract interface
    let app = AppInterface::new(APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;

    abstr_deployment.ans_host.execute(
        &abstract_core::ans_host::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![(DENOM.to_owned(), AssetInfo::native(DENOM).into())],
            to_remove: vec![],
        },
        None,
    )?;

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: ADMIN.to_string(),
            })?;

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(AccountId::local(1), "my-namespace".to_string())?;

    app.deploy(APP_VERSION.parse()?, DeployStrategy::Try)?;

    account.install_app(
        app.clone(),
        &AppInstantiateMsg {
            price_per_minute: Uint128::from(1u128),
            denom: AssetEntry::from(DENOM),
            utc_offset: 0,
            start_time: Time { hour: 9, minute: 0 },
            end_time: Time {
                hour: 17,
                minute: 0,
            },
        },
        None,
    )?;

    Ok((account, abstr_deployment, app, mock))
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, app, _mock) = setup()?;

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
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::MeetingConflictExists {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_contained_in_another() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::MeetingConflictExists {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_left_intersection() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::MeetingConflictExists {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_right_intersection() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::MeetingConflictExists {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_in_past() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::StartTimeMustBeInFuture {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_end_time_before_start_time() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::EndTimeMustBeAfterStartTime {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_start_time_out_of_calendar_bounds() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::StartTimeDoesNotFallWithinCalendarBounds {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_end_time_out_of_calendar_bounds() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::EndTimeDoesNotFallWithinCalendarBounds {}.to_string(),
        error.root_cause().to_string()
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_start_and_end_being_on_different_days() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
            &abstract_core::base::ExecuteMsg::Module(AppExecuteMsg::RequestMeeting {
                start_time: meeting_start_datetime.timestamp().into(),
                end_time: meeting_end_datetime.timestamp().into(),
            }),
            Some(&[Coin::new(60, DENOM)]),
        )
        .unwrap_err()
        .into();

    assert_eq!(
        AppError::StartAndEndTimeNotOnSameDay {}.to_string(),
        error.root_cause().to_string(),
    );

    Ok(())
}

#[test]
fn cannot_request_meeting_with_insufficient_funds() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        AppError::InvalidStakeAmountSent {
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
    let (account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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

    mock.wait_blocks(100000)?;

    let day_datetime = meeting_start_datetime
        .date()
        .and_time(NaiveTime::default())
        .timestamp();

    app.set_sender(&account.manager.address()?);
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

    assert_eq!(
        Uint128::from(60u128),
        mock.query_balance(&account.proxy.address()?, DENOM)?
    );

    Ok(())
}

#[test]
fn return_stake() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        mock.query_balance(&sender, DENOM)?
    );

    mock.wait_blocks(100000)?;

    let day_datetime = meeting_start_datetime
        .date()
        .and_time(NaiveTime::default())
        .timestamp();

    app.set_sender(&account.manager.address()?);
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
        mock.query_balance(&sender, DENOM)?
    );

    Ok(())
}

#[test]
fn slash_partial_stake() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (account, _abstr, mut app, mock) = setup()?;
    let block_info: BlockInfo = mock.block_info()?;

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
        mock.query_balance(&sender, DENOM)?
    );

    mock.wait_blocks(100000)?;

    let day_datetime = meeting_start_datetime
        .date()
        .and_time(NaiveTime::default())
        .timestamp();

    app.set_sender(&account.manager.address()?);
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
        mock.query_balance(&sender, DENOM)?
    );

    assert_eq!(
        Uint128::from(20u128),
        mock.query_balance(&account.proxy.address()?, DENOM)?
    );

    Ok(())
}
