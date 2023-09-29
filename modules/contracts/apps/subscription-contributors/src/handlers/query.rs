use crate::contract::{App, AppResult};
use crate::msg::{AppQueryMsg, ConfigResponse};
use crate::state::CONTRIBUTION_CONFIG;
use abstract_sdk::AccountVerification;
use abstract_subscription_interface::contributors::msg::{ContributorStateResponse, StateResponse};
use abstract_subscription_interface::contributors::state::{CONTRIBUTION_STATE, CONTRIBUTORS};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};

pub fn query_handler(deps: Deps, _env: Env, app: &App, msg: AppQueryMsg) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_binary(&query_config(deps)?),
        AppQueryMsg::State {} => to_binary(&StateResponse {
            contribution: CONTRIBUTION_STATE.load(deps.storage)?,
        }),
        AppQueryMsg::ContributorState { os_id } => {
            let account_registry = app.account_registry(deps);
            let contributor_addr = account_registry.account_base(&os_id)?.manager;
            let maybe_contributor = CONTRIBUTORS.may_load(deps.storage, &contributor_addr)?;
            let subscription_state = if let Some(compensation) = maybe_contributor {
                to_binary(&ContributorStateResponse { compensation })?
            } else {
                return Err(StdError::generic_err("provided address is not a contributor").into());
            };
            Ok(subscription_state)
        }
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONTRIBUTION_CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}
