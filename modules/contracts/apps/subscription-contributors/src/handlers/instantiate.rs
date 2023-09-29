use abstract_subscription_interface::state::contributors::{ContributionState, CONTRIBUTION_STATE};
use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::contract::{App, AppResult};
use crate::msg::ContributorsInstantiateMsg;
use crate::state::{ContributorsConfig, CONTRIBUTION_CONFIG};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: App,
    msg: ContributorsInstantiateMsg,
) -> AppResult {
    let contributor_config: ContributorsConfig = ContributorsConfig {
        emissions_amp_factor: msg.emissions_amp_factor,
        emission_user_share: msg.emission_user_share,
        emissions_offset: msg.emissions_offset,
        protocol_income_share: msg.protocol_income_share,
        max_emissions_multiple: msg.max_emissions_multiple,
        token_info: msg.token_info.check(deps.api, None)?,
    }
    .verify()?;

    let contributor_state: ContributionState = ContributionState {
        income_target: Decimal::zero(),
        expense: Decimal::zero(),
        total_weight: Uint128::zero(),
        emissions: Decimal::zero(),
    };
    CONTRIBUTION_CONFIG.save(deps.storage, &contributor_config)?;
    CONTRIBUTION_STATE.save(deps.storage, &contributor_state)?;

    Ok(Response::new())
}
