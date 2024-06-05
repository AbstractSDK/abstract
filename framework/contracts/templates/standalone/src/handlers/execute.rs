use crate::{
    contract::{MyStandalone, MyStandaloneResult},
    msg::MyStandaloneExecuteMsg,
    state::{CONFIG, COUNT},
    MY_STANDALONE,
};

use abstract_standalone::traits::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MyStandaloneExecuteMsg,
) -> MyStandaloneResult {
    let standalone = MY_STANDALONE;
    match msg {
        MyStandaloneExecuteMsg::UpdateConfig {} => update_config(deps, info, standalone),
        MyStandaloneExecuteMsg::Increment {} => increment(deps, standalone),
        MyStandaloneExecuteMsg::Reset { count } => reset(deps, info, count, standalone),
        MyStandaloneExecuteMsg::IbcCallback(_) => todo!(),
        MyStandaloneExecuteMsg::ModuleIbc(_) => todo!(),
    }
}

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    _info: MessageInfo,
    standalone: MyStandalone,
) -> MyStandaloneResult {
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(standalone.response("update_config"))
}

fn increment(deps: DepsMut, standalone: MyStandalone) -> MyStandaloneResult {
    COUNT.update(deps.storage, |count| MyStandaloneResult::Ok(count + 1))?;

    Ok(standalone.response("increment"))
}

fn reset(
    deps: DepsMut,
    _info: MessageInfo,
    count: i32,
    standalone: MyStandalone,
) -> MyStandaloneResult {
    COUNT.save(deps.storage, &count)?;

    Ok(standalone.response("reset"))
}
