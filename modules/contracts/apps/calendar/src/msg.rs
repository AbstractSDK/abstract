use abstract_core::objects::AssetEntry;
use chrono::NaiveTime;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Int64, Uint128};

use crate::{contract::CalendarApp, error::CalendarError, state::Meeting};

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(CalendarApp, CalendarExecuteMsg, CalendarQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
#[derive(Copy, Eq, PartialOrd)]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
}

impl Time {
    pub fn validate(&self) -> Result<(), CalendarError> {
        if self.hour > 23 {
            return Err(CalendarError::HourOutOfBounds {});
        } else if self.minute > 59 {
            return Err(CalendarError::MinutesOutOfBounds {});
        }
        Ok(())
    }
}

impl From<Time> for NaiveTime {
    fn from(value: Time) -> Self {
        // TODO: handle option
        NaiveTime::from_hms_opt(value.hour, value.minute, 0).unwrap()
    }
}

#[cosmwasm_schema::cw_serde]
pub struct CalendarInstantiateMsg {
    /// The price per minute charged to determine the amount of stake necessary to request a
    /// meeting for a given length.
    pub price_per_minute: Uint128,
    /// The denom of the staked asset.
    pub denom: AssetEntry,
    /// The utc offset of the timezone.
    pub utc_offset: i32,
    /// The start time for each day that meetings can be scheduled.
    pub start_time: Time,
    /// The end time for each day that meetings can be scheduled.
    pub end_time: Time,
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
#[impl_into(ExecuteMsg)]
pub enum CalendarExecuteMsg {
    #[payable]
    /// Request a new meeting.
    RequestMeeting {
        /// The unix timestamp of the start datetime of the meeting.
        start_time: Int64,
        /// The unix timestamp of the end datetime of the meeting.
        end_time: Int64,
    },
    /// Fully slashes the stake for a completed meeting. Admin only.
    SlashFullStake {
        /// The unix timestamp denoting the start of the day the meeting is on. This is equivalent
        /// to the "time" portion being all zero with respect to the `config.utc_offset`.
        day_datetime: Int64,
        /// The index of the meeting to be slashed.
        meeting_index: u32,
    },
    /// Partially slashes the stake for a completed meeting based on how many minutes the requester
    /// was late by. Admin only.
    SlashPartialStake {
        /// The unix timestamp denoting the start of the day the meeting is on. This is equivalent
        /// to the "time" portion being all zero with respect to the `config.utc_offset`.
        day_datetime: Int64,
        /// The index of the meeting to be slashed.
        meeting_index: u32,
        /// The number of minutes the requester was late by resulting in a prorated slash.
        minutes_late: u32,
    },
    /// Returns the full stake for a completed meeting. Admin only.
    ReturnStake {
        /// The unix timestamp denoting the start of the day the meeting is on. This is equivalent
        /// to the "time" portion being all zero with respect to the `config.utc_offset`.
        day_datetime: Int64,
        /// The index of the meeting whose stake should be returned.
        meeting_index: u32,
    },
    /// Update the config. Admin only.
    UpdateConfig {
        /// The updated price per minute.
        price_per_minute: Option<Uint128>,
        /// The updated denom.
        denom: Option<AssetEntry>,
    },
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
#[impl_into(QueryMsg)]
pub enum CalendarQueryMsg {
    /// Returns the config.
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns the meetings for a given day.
    /// Returns [`MeetingsResponse`]
    #[returns(MeetingsResponse)]
    Meetings { day_datetime: Int64 },
}

#[cosmwasm_schema::cw_serde]
pub struct CalendarMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub price_per_minute: Uint128,
    pub utc_offset: i32,
    pub start_time: Time,
    pub end_time: Time,
}

#[cosmwasm_schema::cw_serde]
pub struct MeetingsResponse {
    pub meetings: Vec<Meeting>,
}
