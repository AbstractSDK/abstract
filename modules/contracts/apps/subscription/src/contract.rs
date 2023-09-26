use crate::commands::BLOCKS_PER_MONTH;
use crate::commands::{self, receive_cw20};
use crate::error::SubscriptionError;
use crate::msg::{
    ConfigResponse, ContributorStateResponse, InstantiateMsg, MigrateMsg, StateResponse,
    SubscriberStateResponse, SubscriptionFeeResponse,
};
use crate::msg::{SubscriptionExecuteMsg, SubscriptionQueryMsg};
use crate::state::*;
use crate::SUBSCRIPTION;
use abstract_app::AppContract;
use abstract_sdk::AccountVerification;
use cosmwasm_std::{to_binary, Binary, Decimal, StdError, Uint128};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use cw20::Cw20ReceiveMsg;
use cw_asset::Asset;

pub type SubscriptionResult<T = Response> = Result<T, SubscriptionError>;

pub type SubscriptionApp = AppContract<
    SubscriptionError,
    InstantiateMsg,
    SubscriptionExecuteMsg,
    SubscriptionQueryMsg,
    MigrateMsg,
    Cw20ReceiveMsg,
>;

const SUBSCRIPTION_MODULE: SubscriptionApp =
    SubscriptionApp::new(SUBSCRIPTION, CONTRACT_VERSION, None)
        .with_execute(request_handler)
        .with_instantiate(instantiate_handler)
        .with_query(query_handler)
        .with_receive(receive_cw20);
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// export endpoints
#[cfg(feature = "export")]
abstract_app::export_endpoints!(SUBSCRIPTION_MODULE, SubscriptionApp);

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _app: SubscriptionApp,
    msg: InstantiateMsg,
) -> SubscriptionResult {
    let subscription_config: SubscriptionConfig = SubscriptionConfig {
        payment_asset: msg.subscription.payment_asset.check(deps.api, None)?,
        subscription_cost_per_block: msg.subscription.subscription_cost_per_block,
        factory_address: deps.api.addr_validate(&msg.subscription.factory_addr)?,
        subscription_per_block_emissions: msg
            .subscription
            .subscription_per_block_emissions
            .check(deps.api)?,
    };

    let subscription_state: SubscriptionState = SubscriptionState { active_subs: 0 };

    // Optional contribution setup
    if let Some(msg) = msg.contribution {
        let contributor_config: ContributionConfig = ContributionConfig {
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
        INCOME_TWA.instantiate(deps.storage, &env, None, msg.income_averaging_period.u64())?;
    }

    SUBSCRIPTION_CONFIG.save(deps.storage, &subscription_config)?;
    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;

    Ok(Response::new())
}

fn request_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: SubscriptionApp,
    msg: SubscriptionExecuteMsg,
) -> SubscriptionResult {
    match msg {
        SubscriptionExecuteMsg::Pay { os_id } => {
            let maybe_received_coin = info.funds.last();
            if let Some(coin) = maybe_received_coin.cloned() {
                commands::try_pay(app, deps, env, info, Asset::from(coin), os_id)
            } else {
                Err(SubscriptionError::NotUsingCW20Hook {})
            }
        }
        SubscriptionExecuteMsg::Unsubscribe { os_ids } => {
            commands::unsubscribe(deps, env, app, os_ids)
        }
        SubscriptionExecuteMsg::ClaimCompensation { os_id } => {
            commands::try_claim_compensation(app, deps, env, os_id)
        }
        SubscriptionExecuteMsg::ClaimEmissions { os_id } => {
            commands::claim_subscriber_emissions(&app, deps.as_ref(), &env, os_id)
        }
        SubscriptionExecuteMsg::UpdateContributor {
            contributor_os_id,
            base_per_block,
            weight,
            expiration_block,
        } => commands::update_contributor_compensation(
            deps,
            env,
            info,
            app,
            contributor_os_id,
            base_per_block,
            weight.map(|w| w.u64() as u32),
            expiration_block.map(|w| w.u64()),
        ),
        SubscriptionExecuteMsg::RemoveContributor { os_id } => {
            commands::remove_contributor(deps, info, os_id)
        }
        SubscriptionExecuteMsg::UpdateSubscriptionConfig {
            payment_asset,
            version_control_address,
            factory_address,
            subscription_cost,
        } => commands::update_subscription_config(
            deps,
            env,
            info,
            payment_asset,
            version_control_address,
            factory_address,
            subscription_cost,
        ),
        SubscriptionExecuteMsg::UpdateContributionConfig {
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token_info,
            emissions_amp_factor,
            emissions_offset,
        } => commands::update_contribution_config(
            deps,
            env,
            info,
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token_info,
            emissions_amp_factor,
            emissions_offset,
        ),
    }
}

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &SubscriptionApp,
    msg: SubscriptionQueryMsg,
) -> SubscriptionResult<Binary> {
    match msg {
        // handle dapp-specific queries here
        SubscriptionQueryMsg::State {} => {
            let subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
            let contributor_state = CONTRIBUTION_STATE.load(deps.storage)?;
            to_binary(&StateResponse {
                contribution: contributor_state,
                subscription: subscription_state,
            })
        }
        SubscriptionQueryMsg::Fee {} => {
            let config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            let minimal_cost = Uint128::from(BLOCKS_PER_MONTH) * config.subscription_cost_per_block;
            to_binary(&SubscriptionFeeResponse {
                fee: Asset {
                    info: config.payment_asset,
                    amount: minimal_cost,
                },
            })
        }
        SubscriptionQueryMsg::Config {} => {
            let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            let contributor_config = CONTRIBUTION_CONFIG.load(deps.storage)?;
            to_binary(&ConfigResponse {
                contribution: contributor_config,
                subscription: subscription_config,
            })
        }
        SubscriptionQueryMsg::SubscriberState { os_id } => {
            let maybe_sub = SUBSCRIBERS.may_load(deps.storage, os_id)?;
            let maybe_dormant_sub = DORMANT_SUBSCRIBERS.may_load(deps.storage, os_id)?;
            let subscription_state = if let Some(sub) = maybe_sub {
                to_binary(&SubscriberStateResponse {
                    currently_subscribed: true,
                    subscriber_details: sub,
                })?
            } else if let Some(sub) = maybe_dormant_sub {
                to_binary(&SubscriberStateResponse {
                    currently_subscribed: true,
                    subscriber_details: sub,
                })?
            } else {
                return Err(StdError::generic_err("os has os_id 0 or does not exist").into());
            };
            Ok(subscription_state)
        }
        SubscriptionQueryMsg::ContributorState { os_id } => {
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
