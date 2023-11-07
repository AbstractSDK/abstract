use abstract_app::sdk::features::AbstractNameService;
use abstract_app::sdk::AbstractSdkError;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{CroncatApp, CroncatResult};
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG};
use crate::utils;

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: CroncatApp,
    _msg: AppInstantiateMsg,
) -> CroncatResult {
    CONFIG.save(deps.storage, &Config {})?;

    utils::factory_addr(&deps.querier, &app.ans_host(deps.as_ref())?).map_err(|err| {
        AbstractSdkError::generic_err(format!("Cron Cat Factory not found in ANS: {err:?}"))
    })?;
    Ok(Response::new())
}
