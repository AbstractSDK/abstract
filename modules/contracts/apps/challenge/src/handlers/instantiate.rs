use crate::contract::{AppResult, ChallengeApp};
use crate::msg::AppInstantiateMsg;
use crate::state::NEXT_ID;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: ChallengeApp,
    msg: AppInstantiateMsg,
) -> AppResult {
    NEXT_ID.save(deps.storage, &0)?;
    Ok(Response::new())
}
