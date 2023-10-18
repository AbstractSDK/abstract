use crate::contract::{AppResult, ContributorsApp};
use crate::msg::{ContributorStateResponse, ContributorsQueryMsg, StateResponse};
use crate::state::{CONTRIBUTION_CONFIG, CONTRIBUTION_STATE, CONTRIBUTORS};
use abstract_sdk::AccountVerification;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &ContributorsApp,
    msg: ContributorsQueryMsg,
) -> AppResult<Binary> {
    match msg {
        ContributorsQueryMsg::Config {} => to_binary(&CONTRIBUTION_CONFIG.load(deps.storage)?),
        ContributorsQueryMsg::State {} => to_binary(&StateResponse {
            contribution: CONTRIBUTION_STATE.load(deps.storage)?,
        }),
        ContributorsQueryMsg::ContributorState { os_id } => {
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
