use chrono::{DateTime, FixedOffset, NaiveTime, Timelike};
use cosmwasm_std::{Addr, BlockInfo, Uint128};
use cw_storage_plus::{Item, Map};

use crate::{error::CalendarError, msg::Time};

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub price_per_minute: Uint128,
    pub denom: String,
    pub utc_offset: i32,
    pub start_time: Time,
    pub end_time: Time,
}

#[cosmwasm_schema::cw_serde]
pub struct Meeting {
    pub start_time: i64,
    pub end_time: i64,
    pub requester: Addr,
    pub amount_staked: Uint128,
}

impl Meeting {
    pub fn new(
        config: &Config,
        block: &BlockInfo,
        meeting_start_datetime: DateTime<FixedOffset>,
        meeting_end_datetime: DateTime<FixedOffset>,
        requester: Addr,
        amount_staked: Uint128,
    ) -> Result<Self, CalendarError> {
        let meeting_start_timestamp = meeting_start_datetime.timestamp();
        let meeting_end_timestamp = meeting_end_datetime.timestamp();

        let meeting_start_time: NaiveTime = meeting_start_datetime.time();
        let meeting_end_time: NaiveTime = meeting_end_datetime.time();

        let calendar_start_time: NaiveTime = config.start_time.into();
        let calendar_end_time: NaiveTime = config.end_time.into();

        if meeting_start_datetime.date_naive() != meeting_end_datetime.date_naive() {
            return Err(CalendarError::StartAndEndTimeNotOnSameDay {});
        }

        if meeting_start_time.second() != 0 || meeting_start_time.nanosecond() != 0 {
            return Err(CalendarError::StartTimeNotRoundedToNearestMinute {});
        }

        if meeting_end_time.second() != 0 || meeting_end_time.nanosecond() != 0 {
            return Err(CalendarError::EndTimeNotRoundedToNearestMinute {});
        }

        // Not 100% sure about this typecasting but the same is done in the cosmwasm doc example using
        // chrono so it should be fine.
        if (block.time.seconds() as i64) > meeting_start_timestamp {
            return Err(CalendarError::StartTimeMustBeInFuture {});
        }

        if meeting_start_time >= meeting_end_time {
            return Err(CalendarError::EndTimeMustBeAfterStartTime {});
        }

        if meeting_start_time < calendar_start_time || meeting_start_time > calendar_end_time {
            return Err(CalendarError::OutOfBoundsStartTime {});
        }

        if meeting_end_time < calendar_start_time || meeting_end_time > calendar_end_time {
            return Err(CalendarError::OutOfBoundsEndTime {});
        }

        Ok(Meeting {
            start_time: meeting_start_timestamp,
            end_time: meeting_end_timestamp,
            requester,
            amount_staked,
        })
    }
}

// unix start-time of the day -> vector of meetings in that day.
pub const CALENDAR: Map<i64, Vec<Meeting>> = Map::new("calendar");
pub const CONFIG: Item<Config> = Item::new("config");
