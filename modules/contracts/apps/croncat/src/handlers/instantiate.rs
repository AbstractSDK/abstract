use abstract_sdk::{features::AbstractNameService, AbstractSdkError};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{CroncatApp, CroncatResult},
    msg::AppInstantiateMsg,
    state::{Config, CONFIG},
    utils,
};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: CroncatApp,
    _msg: AppInstantiateMsg,
) -> CroncatResult {
    CONFIG.save(deps.storage, &Config {})?;

    let name_service = app.name_service(deps.as_ref());
    utils::factory_addr(&name_service).map_err(|err| {
        AbstractSdkError::generic_err(format!("Cron Cat Factory not found in ANS: {err:?}"))
    })?;
    Ok(Response::new())
}
