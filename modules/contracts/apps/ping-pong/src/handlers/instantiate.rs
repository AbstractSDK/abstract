use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{App, AppResult},
    msg::AppInstantiateMsg,
    state::{ICS20_CALLBACKS, LOSSES, WINS},
};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _module: App,
    _msg: AppInstantiateMsg,
) -> AppResult {
    WINS.save(deps.storage, &0)?;
    LOSSES.save(deps.storage, &0)?;
    ICS20_CALLBACKS.save(deps.storage, &vec![])?;

    Ok(Response::new())
}
