use crate::contract::{App, AppResult};
use crate::msg::{AppQueryMsg, ConfigResponse, MeetingsResponse};
use crate::state::{CALENDAR, CONFIG};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Int64, StdResult};

pub fn query_handler(deps: Deps, _env: Env, _app: &App, msg: AppQueryMsg) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_binary(&query_config(deps)?),
        AppQueryMsg::Meetings { day_datetime } => to_binary(&query_meetings(deps, day_datetime)?),
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
