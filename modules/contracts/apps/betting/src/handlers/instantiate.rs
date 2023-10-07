use abstract_core::objects::fee::Fee;
use abstract_sdk::AbstractResponse;
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response};

use crate::contract::{BetApp, BetResult};
use crate::msg::BetInstantiateMsg;
use crate::state::{Config, CONFIG, DEFAULT_RAKE_PERCENT, State, STATE};

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: BetApp,
    msg: BetInstantiateMsg,
) -> BetResult {
    let state: State = State::default();
    STATE.save(deps.storage, &state)?;

    let config = Config {
        rake: Fee::new(msg.rake.unwrap_or(Decimal::percent(DEFAULT_RAKE_PERCENT)))?,
    };

    config.validate(deps.as_ref())?;
    CONFIG.save(deps.storage, &config)?;

    Ok(app.tag_response(Response::new(), "instantiate"))
}
