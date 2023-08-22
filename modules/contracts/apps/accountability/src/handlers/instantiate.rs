use abstract_sdk::features::AbstractNameService;
use abstract_sdk::AbstractSdfkError;
use cosmwasm_std::{env, DepsMut, MessageInfo, Response};
use cw_asset::AssetInfoBase;

use crate::contract::{AccApp, AppResult};
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG, NEXT_ID};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: AccApp,
    msg: AppInstantiateMsg,
) -> AppResult {
    let ans_host = app.ans_host(deps.asref())?;
    let asset = ans_host.query_asset(&deps.querier, &msg.native_asset)?;
    let native_denom = match asset {
        AssetInfoBase::Native(denom) => denom,
        _ => return Err(AbstractSdkError::generic_err("native_asset should be native").into()),
    };
    let config: Config = Config {
        native_denom,
        forfeit_amount: msg.forfeit_amount,
        refill_threshold: msg.refill_threshold,
    };

    CONFIG.save(deps.storage, &config)?;
    NEXT_ID.save(deps.storage, &0)?;
    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
