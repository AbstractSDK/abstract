use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use cw_asset::AssetInfo;

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

    let ans = app.name_service(deps.as_ref());

    if let Some(asset) = msg.desired_asset.clone() {
        ans.query(&asset).map_err(|_| AppError::DesiredAssetDoesNotExist {})?;
    }

    let config: Config = Config {
        desired_asset: msg.desired_asset,
        exchanges: msg.exchanges,
    };

    CONFIG.save(deps.storage, &config)?;

    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
