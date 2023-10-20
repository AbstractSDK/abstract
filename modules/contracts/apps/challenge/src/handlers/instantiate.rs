use crate::contract::{AppResult, ChallengeApp};
use crate::msg::ChallengeInstantiateMsg;
use crate::state::{NEXT_ID, SIMPLE_VOTING};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: ChallengeApp,
    msg: ChallengeInstantiateMsg,
) -> AppResult {
    NEXT_ID.save(deps.storage, &0u64)?;
    SIMPLE_VOTING.instantiate(deps.storage, &msg.vote_config)?;
    Ok(Response::new())
}
