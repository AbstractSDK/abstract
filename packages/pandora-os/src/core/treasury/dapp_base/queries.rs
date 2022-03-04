use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

use crate::core::treasury::dapp_base::msg::{BaseQueryMsg, BaseStateResponse};
use crate::core::treasury::dapp_base::state::BASESTATE;

/// Handles the common base queries
pub fn handle_base_query(deps: Deps, query: BaseQueryMsg) -> StdResult<Binary> {
    match query {
        BaseQueryMsg::Config {} => to_binary(&try_query_config(deps)?),
    }
}
/// Returns the BaseState
pub fn try_query_config(deps: Deps) -> StdResult<BaseStateResponse> {
    let state = BASESTATE.load(deps.storage)?;

    Ok(BaseStateResponse {
        treasury_address: state.treasury_address.into_string(),
        traders: state.traders.into_iter().map(|t| t.into_string()).collect(),
        memory_address: state.memory.address.into_string(),
    })
}
