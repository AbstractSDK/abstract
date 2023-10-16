use abstract_core::objects::AssetEntry;
use chrono::NaiveTime;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Int64, Uint128};

use crate::{contract::App, state::Meeting};

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(App, AppExecuteMsg, AppQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
}

impl From<Time> for NaiveTime {
    fn from(value: Time) -> Self {
        // TODO: handle option
        NaiveTime::from_hms_opt(value.hour, value.minute, 0).unwrap()
    }
}

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    pub price_per_minute: Uint128,
    pub denom: AssetEntry,
    pub utc_offset: i32,
    pub start_time: Time,
    pub end_time: Time,
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    #[cfg_attr(feature = "interface", payable)]
    RequestMeeting { start_time: Int64, end_time: Int64 },
    SlashFullStake {
        day_datetime: Int64,
        meeting_index: u32,
    },
    SlashPartialStake {
        day_datetime: Int64,
        meeting_index: u32,
        minutes_late: u32,
    },
    ReturnStake {
        day_datetime: Int64,
        meeting_index: u32,
    },
    UpdateConfig {
        price_per_minute: Option<Uint128>,
        denom: Option<AssetEntry>,
    },
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum AppQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(MeetingsResponse)]
    Meetings { datetime: i64 },
}

#[cosmwasm_schema::cw_serde]
pub enum AppMigrateMsg {}

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
