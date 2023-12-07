use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{App, AppResult};
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG, COUNT};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: App,
    msg: AppInstantiateMsg,
) -> AppResult {
    let config: Config = Config {};

    CONFIG.save(deps.storage, &config)?;
    COUNT.save(deps.storage, &msg.count)?;

    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
