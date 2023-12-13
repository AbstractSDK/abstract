use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{CalendarApp, CalendarAppResult};
use crate::error::CalendarError;
use crate::msg::CalendarInstantiateMsg;
use crate::state::{Config, CONFIG};

use super::execute::resolve_native_ans_denom;

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: CalendarApp,
    msg: CalendarInstantiateMsg,
) -> CalendarAppResult {
    let denom = resolve_native_ans_denom(deps.as_ref(), &app, msg.denom)?;

    msg.start_time.validate()?;
    msg.end_time.validate()?;

    if msg.start_time >= msg.end_time {
        return Err(CalendarError::EndTimeMustBeAfterStartTime {});
    }

    let config: Config = Config {
        price_per_minute: msg.price_per_minute,
        denom,
        utc_offset: msg.utc_offset,
        start_time: msg.start_time,
        end_time: msg.end_time,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}
