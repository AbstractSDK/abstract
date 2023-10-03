use crate::state::{ContributionState, CONTRIBUTION_STATE};
use abstract_sdk::ModuleInterface;
use abstract_subscription_interface::SUBSCRIPTION_ID;
use cosmwasm_std::{wasm_execute, Decimal, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::contract::{ContributorsApp, AppResult};
use crate::msg::ContributorsInstantiateMsg;
use crate::state::{ContributorsConfig, CONTRIBUTION_CONFIG};

use abstract_subscription_interface::subscription::msg as subscr_msg;

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: ContributorsApp,
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

    // TODO: this contract is not installed yet and reading module-factory context sounds wrong
    // come up with a better solution something like post-install hooks to abstract
    //
    // self-enable contributors
    // let subscription_addr = app.modules(deps.as_ref()).module_address(SUBSCRIPTION_ID)?;
    // let update_config_msg = wasm_execute(
    //     subscription_addr,
    //     &subscr_msg::ExecuteMsg::from(
    //         subscr_msg::SubscriptionExecuteMsg::UpdateSubscriptionConfig {
    //             payment_asset: None,
    //             factory_address: None,
    //             subscription_cost_per_week: None,
    //             contributors_enabled: Some(true),
    //         },
    //     ),
    //     vec![],
    // )?;
    Ok(Response::new())
}
