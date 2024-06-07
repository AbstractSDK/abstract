use crate::{
    contract::MyStandaloneResult,
    msg::MyStandaloneInstantiateMsg,
    state::{Config, CONFIG, COUNT},
    MY_STANDALONE,
};

use abstract_standalone::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: MyStandaloneInstantiateMsg,
) -> MyStandaloneResult {
    let config: Config = Config {};
    CONFIG.save(deps.storage, &config)?;
    COUNT.save(deps.storage, &msg.count)?;

    // Init standalone as module
    let is_migratable = true;
    MY_STANDALONE.instantiate(deps.branch(), &env, msg.base, is_migratable)?;

    Ok(MY_STANDALONE.response("init"))
}
