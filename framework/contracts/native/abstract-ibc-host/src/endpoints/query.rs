use abstract_core::{ibc_host::ConfigResponse, objects::AccountId};
use abstract_sdk::{
    base::{Handler, QueryEndpoint},
    core::ibc_host::QueryMsg,
};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};

use crate::state::CONFIG;

pub fn query(deps: Deps, _env: Env, query: QueryMsg) -> StdResult<Binary> {
    match query {
        QueryMsg::Config {} => to_binary(&dapp_config(deps)?),
    }
}
fn dapp_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        ans_host_address: state.ans_host.address,
        account_factory_address: state.account_factory,
        version_control_address: state.version_control,
    })
}
