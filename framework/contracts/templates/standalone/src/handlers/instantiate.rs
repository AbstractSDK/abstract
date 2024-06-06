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
    _env: Env,
    _info: MessageInfo,
    msg: MyStandaloneInstantiateMsg,
) -> MyStandaloneResult {
    let config: Config = Config {};
    CONFIG.save(deps.storage, &config)?;
    COUNT.save(deps.storage, &msg.count)?;

    // Init standalone as module
    MY_STANDALONE.instantiate(
        deps.branch(),
        msg.base.expect("Module factory should fill this"),
    )?;

    Ok(MY_STANDALONE.response("init"))
}
