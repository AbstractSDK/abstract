use abstract_sdk::features::AbstractNameService;
use abstract_sdk::{AbstractSdkError, ModuleRegistryInterface};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw_asset::AssetInfoBase;

use crate::contract::{AppResult, DCAApp};
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
        _ => return Err(AbstractSdkError::generic_err("native_asset should be native").into()),
    };
    let config: Config = Config {
        native_denom,
        dca_creation_amount: msg.dca_creation_amount,
        refill_threshold: msg.refill_threshold,
        max_spread: msg.max_spread,
    };

    CONFIG.save(deps.storage, &config)?;
    NEXT_ID.save(deps.storage, &0)?;
    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
