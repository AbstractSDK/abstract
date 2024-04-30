use abstract_app::sdk::AbstractSdkError;
use abstract_app::std::AbstractError;
use abstract_app::AppError as AbstractAppError;
use cosmwasm_std::{StdError, Uint128};
use cw_asset::AssetError;
use cw_controllers::AdminError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CalendarError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    DappError(#[from] AbstractAppError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Start time must be in future")]
    StartTimeMustBeInFuture {},

    #[error("Start time does not fall within calendar bounds")]
    OutOfBoundsStartTime {},

    #[error("End time does not fall within calendar bounds")]
    OutOfBoundsEndTime {},

    #[error("End time must be after start time")]
    EndTimeMustBeAfterStartTime {},

    #[error("Meeting conflict exists")]
    MeetingConflictExists {},

    #[error("Invalid time")]
    InvalidTime {},

    #[error("Start and end time not on same day")]
    StartAndEndTimeNotOnSameDay {},

    #[error("Start time not rounded to nearest minute")]
    StartTimeNotRoundedToNearestMinute {},

    #[error("End time not rounded to nearest minute")]
    EndTimeNotRoundedToNearestMinute {},

    #[error("Invalid stack amount sent. Expected_amount: {expected_amount}")]
    InvalidStakeAmountSent { expected_amount: Uint128 },

    #[error("No meetings at given day datetime")]
    NoMeetingsAtGivenDayDateTime {},

    #[error("Meeting does not exist")]
    MeetingDoesNotExist {},

    #[error("Meeting not finished yet")]
    MeetingNotFinishedYet {},

    #[error("Stake already handled")]
    StakeAlreadyHandled {},

    #[error("Minutes late cannot exceed duration of meeting")]
    MinutesLateCannotExceedDurationOfMeeting {},

    #[error("Invalid utc offset")]
    InvalidUtcOffset {},

    #[error("Hour out of bounds")]
    HourOutOfBounds {},

    #[error("Minutes out of bounds")]
    MinutesOutOfBounds {},
}
