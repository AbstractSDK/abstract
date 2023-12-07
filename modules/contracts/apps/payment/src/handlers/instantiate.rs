use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{AppResult, PaymentApp};
use crate::error::AppError;
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: PaymentApp,
    msg: AppInstantiateMsg,
) -> AppResult {
    let name_service = app.name_service(deps.as_ref());

    if let Some(asset) = &msg.desired_asset {
        name_service
            .query(asset)
            .map_err(|_| AppError::DesiredAssetDoesNotExist {})?;
    }
    let ans_dexes = name_service.registered_dexes()?;
    for dex in msg.exchanges.iter() {
        if !ans_dexes.dexes.contains(dex) {
            return Err(AppError::DexNotRegistered(dex.to_owned()));
        }
    }

    let config: Config = Config {
        desired_asset: msg.desired_asset,
        exchanges: msg.exchanges,
    };

    CONFIG.save(deps.storage, &config)?;

    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
