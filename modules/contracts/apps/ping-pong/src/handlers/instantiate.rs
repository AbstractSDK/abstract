use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{App, AppResult};
use crate::msg::AppInstantiateMsg;
use crate::state::WINS;

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: App,
    _msg: AppInstantiateMsg,
) -> AppResult {
    WINS.save(deps.storage, &0)?;

    Ok(Response::new())
}
