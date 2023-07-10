use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{AppResult, DCAApp};
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG, NEXT_ID};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: DCAApp,
    msg: AppInstantiateMsg,
) -> AppResult {
    let config: Config = Config {
        native_denom: msg.native_denom,
        dca_creation_amount: msg.dca_creation_amount,
        refill_threshold: msg.refill_threshold,
        max_spread: msg.max_spread,
    };

    CONFIG.save(deps.storage, &config)?;
    NEXT_ID.save(deps.storage, &0)?;
    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
