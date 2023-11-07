use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use super::super::contract::{App, AppResult};
use super::super::msg::AppInstantiateMsg;
use super::super::state::{Config, CONFIG};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: App,
    _msg: AppInstantiateMsg,
) -> AppResult {
    let config: Config = Config {};

    CONFIG.save(deps.storage, &config)?;

    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
