use cosmwasm_std::{Binary, Deps, Env, Order, StdResult, to_binary};

use crate::contract::{AppResult, DiceApp};
use crate::msg::DiceQueryMsg;
use crate::state::DOUBLE_DICE_OUTCOME;

pub fn query_handler(deps: Deps, _env: Env, app: &DiceApp, msg: DiceQueryMsg) -> AppResult<Binary> {
    match msg {
        DiceQueryMsg::GetHistoryOfRounds {} => to_binary(&query_history(deps)?),
        DiceQueryMsg::QueryOutcome { job_id } => to_binary(&query_outcome(deps, job_id)?),
    }
    .map_err(Into::into)
}

//Query the outcome for a sepcific dice roll round/job_id
fn query_outcome(deps: Deps, job_id: String) -> StdResult<Option<u8>> {
    let outcome = DOUBLE_DICE_OUTCOME.may_load(deps.storage, &job_id)?;
    Ok(outcome)
}

//This function shows all the history of the dice outcomes from all rounds/job_ids
fn query_history(deps: Deps) -> StdResult<Vec<String>> {
    let out: Vec<String> = DOUBLE_DICE_OUTCOME
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(id, value)| format!("{id}:{value}")))
        .collect::<StdResult<_>>()?;
    Ok(out)
}