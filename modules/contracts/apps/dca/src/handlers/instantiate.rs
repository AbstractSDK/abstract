use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw_asset::AssetInfoBase;

use crate::contract::{AppResult, DCAApp};
use crate::error::DCAError;
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG, NEXT_ID};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: DCAApp,
    msg: AppInstantiateMsg,
) -> AppResult {
    let ans_host = app.ans_host(deps.as_ref())?;
    let asset = ans_host.query_asset(&deps.querier, &msg.native_asset)?;
    let native_denom = match asset {
        AssetInfoBase::Native(denom) => denom,
        _ => return Err(DCAError::NotNativeAsset {}),
    };
    let config: Config = Config {
        native_denom,
        dca_creation_amount: msg.dca_creation_amount,
        refill_threshold: msg.refill_threshold,
        max_spread: msg.max_spread,
    };

    CONFIG.save(deps.storage, &config)?;
    NEXT_ID.save(deps.storage, &Default::default())?;

    Ok(Response::new())
}
