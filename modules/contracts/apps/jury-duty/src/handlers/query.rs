use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use crate::contract::{AppResult, JuryDutyApp};
use crate::msg::{JuryDutyQueryMsg, JuryResponse};
use crate::state::JURIES;

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &JuryDutyApp,
    msg: JuryDutyQueryMsg,
) -> AppResult<Binary> {
    match msg {
        // JuryDutyQueryMsg::Cw3Query(msg) => cw3_fixed_multisig::contract::query(deps, _env, msg),
        JuryDutyQueryMsg::Jury { proposal_id } => to_binary(&query_jury(deps, app, proposal_id)?),
    }
    .map_err(Into::into)
}

fn query_jury(deps: Deps, _app: &JuryDutyApp, proposal_id: u64) -> StdResult<JuryResponse> {
    let jury = JURIES.may_load(deps.storage, &proposal_id)?;

    Ok(JuryResponse { jury })
}
