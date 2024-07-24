use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{AppResult, ChallengeApp},
    msg::ChallengeInstantiateMsg,
    state::{NEXT_ID, SIMPLE_VOTING},
};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _module: ChallengeApp,
    msg: ChallengeInstantiateMsg,
) -> AppResult {
    NEXT_ID.save(deps.storage, &0u64)?;
    SIMPLE_VOTING.instantiate(deps.storage, &msg.vote_config)?;
    Ok(Response::new())
}
