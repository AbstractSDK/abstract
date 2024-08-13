use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Int64, StdResult};

use crate::{
    contract::{CalendarApp, CalendarAppResult},
    msg::{CalendarQueryMsg, ConfigResponse, MeetingsResponse},
    state::{CALENDAR, CONFIG},
};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _module: &CalendarApp,
    msg: CalendarQueryMsg,
) -> CalendarAppResult<Binary> {
    match msg {
        CalendarQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        CalendarQueryMsg::Meetings { day_datetime } => {
            to_json_binary(&query_meetings(deps, day_datetime)?)
        }
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        price_per_minute: config.price_per_minute,
        utc_offset: config.utc_offset,
        start_time: config.start_time,
        end_time: config.end_time,
    })
}

fn query_meetings(deps: Deps, day_datetime: Int64) -> StdResult<MeetingsResponse> {
    let meetings = CALENDAR
        .may_load(deps.storage, day_datetime.i64())?
        .unwrap_or_default();
    Ok(MeetingsResponse { meetings })
}
