use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{AppResult, PaymentApp};
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: PaymentApp,
    msg: AppInstantiateMsg,
) -> AppResult {
    let config: Config = Config {
        desired_asset: msg.desired_asset,
        exchanges: msg.exchanges,
    };

    CONFIG.save(deps.storage, &config)?;

    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
