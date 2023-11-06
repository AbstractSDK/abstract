use cosmwasm_std::{Deps, StdResult};

use super::msg::GetCountResponse;
use super::state::STATE;

pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetCountResponse { count: state.count })
}
