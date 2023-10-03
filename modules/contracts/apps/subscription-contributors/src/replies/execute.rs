use crate::{
    contract::{ContributorsApp, AppResult},
    handlers::execute::{claim_compensation, subscription_module_addr},
};

use abstract_subscription_interface::contributors::state::{
    ContributionState, ContributorsConfig, CACHED_CONTRIBUTION_STATE, COMPENSATION_CLAIMER,
    CONTRIBUTION_CONFIG, CONTRIBUTION_STATE,
};
use abstract_subscription_interface::subscription::state as subscr_state;
use cosmwasm_std::{Decimal, DepsMut, Env, Reply, StdError, StdResult, Storage};

pub fn refresh_reply(deps: DepsMut, env: Env, app: ContributorsApp, _reply: Reply) -> AppResult {
    let config = CONTRIBUTION_CONFIG.load(deps.storage)?;
    let mut state = CONTRIBUTION_STATE.load(deps.storage)?;
    let os_id = COMPENSATION_CLAIMER.load(deps.storage)?;

    let subscription_addr = subscription_module_addr(&app, deps.as_ref())?;
    let income_twa = subscr_state::INCOME_TWA.query(&deps.querier, subscription_addr.clone())?;

    // Cache current state. This state will be used to pay out contributors of last period.
    CACHED_CONTRIBUTION_STATE.save(deps.storage, &state)?;
    // Overwrite current state with new income.
    update_contribution_state(
        deps.storage,
        env,
        &mut state,
        &config,
        income_twa.average_value,
    )?;
    claim_compensation(deps, app, income_twa, os_id)
}

/// Update the contribution state
/// Call when income,target or config changes
fn update_contribution_state(
    store: &mut dyn Storage,
    _env: Env,
    contributor_state: &mut ContributionState,
    contributor_config: &ContributorsConfig,
    income: Decimal,
) -> StdResult<()> {
    let floor_emissions: Decimal =
        (Decimal::from_atomics(contributor_config.emissions_amp_factor, 0)
            .map_err(|e| StdError::GenericErr { msg: e.to_string() })?
            / contributor_state.income_target)
            + Decimal::from_atomics(contributor_config.emissions_offset, 0)
                .map_err(|e| StdError::GenericErr { msg: e.to_string() })?;
    let max_emissions = floor_emissions * contributor_config.max_emissions_multiple;
    if income < contributor_state.income_target {
        contributor_state.emissions = max_emissions
            - (max_emissions - floor_emissions) * (income / contributor_state.income_target);
        contributor_state.expense = income;
    } else {
        contributor_state.expense = contributor_state.income_target;
        contributor_state.emissions = floor_emissions;
    }
    CONTRIBUTION_STATE.save(store, contributor_state)
}
