use crate::contract::{AppResult, ChallengeApp};
use crate::state::NEXT_ID;
use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: ChallengeApp,
    _msg: Empty,
) -> AppResult {
    NEXT_ID.save(deps.storage, &0u64)?;
    Ok(Response::new())
}
