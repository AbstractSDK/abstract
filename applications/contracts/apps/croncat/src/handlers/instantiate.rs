use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{CroncatApp, CroncatResult};
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: CroncatApp,
    _msg: AppInstantiateMsg,
) -> CroncatResult {
    CONFIG.save(deps.storage, &Config {})?;

    Ok(Response::new())
}
