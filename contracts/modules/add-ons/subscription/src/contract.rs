#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use abstract_add_on::AddOnContract;
use abstract_os::registery::SUBSCRIPTION;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_asset::Asset;
use cw_storage_plus::{Endian, Map, U32Key};
use protobuf::Message;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use semver::Version;

use abstract_os::util::fee::Fee;

use crate::error::SubscriptionError;
use crate::{commands, queries};
use abstract_os::modules::add_ons::subscription::msg::{
    ConfigResponse, ContributorStateResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    StateResponse, SubscriberStateResponse, SubscriptionFeeResponse,
};
use abstract_os::modules::add_ons::subscription::state::*;

pub type SubscriptionResult = Result<Response, SubscriptionError>;
pub type SubscriptionAddOn<'a> = AddOnContract<'a>;

const INSTANTIATE_REPLY_ID: u8 = 1u8;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> SubscriptionResult {
    let version = CONTRACT_VERSION.parse::<Version>()?;
    let storage_version = get_contract_version(deps.storage)?
        .version
        .parse::<Version>()?;
    if storage_version < version {
        set_contract_version(deps.storage, SUBSCRIPTION, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> SubscriptionResult {
    let sub_config: SubscriptionConfig = SubscriptionConfig {
        payment_asset: msg.subscription.payment_asset.check(deps.api, None)?,
        subscription_cost: msg.subscription.subscription_cost,
        version_control_address: deps
            .api
            .addr_validate(&msg.subscription.version_control_addr)?,
        factory_address: deps.api.addr_validate(&msg.subscription.factory_addr)?,
    };

    let sub_state: SubscriptionState = SubscriptionState {
        income: Uint64::zero(),
        active_subs: 0,
        collected: false,
    };

    let con_config: ContributionConfig = ContributionConfig {
        project_token: deps.api.addr_validate(&msg.contribution.project_token)?,
        emissions_amp_factor: msg.contribution.emissions_amp_factor,
        emission_user_share: msg.contribution.emission_user_share,
        emissions_offset: msg.contribution.emissions_offset,
        protocol_income_share: msg.contribution.protocol_income_share,
        base_denom: msg.contribution.base_denom,
        max_emissions_multiple: msg.contribution.max_emissions_multiple,
    }
    .verify()?;

    let con_state: ContributionState = ContributionState {
        target: Uint64::zero(),
        expense: Uint64::zero(),
        total_weight: Uint128::zero(),
        emissions: Uint128::zero(),
        next_pay_day: Uint64::from(env.block.time.seconds()),
    };

    SubscriptionAddOn::default().instantiate(
        deps.branch(),
        env,
        info,
        msg.base,
        SUBSCRIPTION,
        CONTRACT_VERSION,
    )?;

    SUB_CONFIG.save(deps.storage, &sub_config)?;
    SUB_STATE.save(deps.storage, &sub_state)?;
    CON_CONFIG.save(deps.storage, &con_config)?;
    CON_STATE.save(deps.storage, &con_state)?;

    CLIENTS.instantiate(deps.storage)?;
    CONTRIBUTORS.instantiate(deps.storage)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> SubscriptionResult {
    let add_on = SubscriptionAddOn::default();

    match msg {
        ExecuteMsg::Base(message) => add_on
            .execute(deps, env, info, message)
            .map_err(|e| e.into()),
        ExecuteMsg::Receive(msg) => commands::receive_cw20(add_on, deps, env, info, msg),
        ExecuteMsg::Pay { os_id } => {
            let maybe_recieved_coin = info.funds.last();
            if let Some(coin) = maybe_recieved_coin.cloned() {
                commands::try_pay(add_on, deps, info, Asset::from(coin), os_id)
            } else {
                Err(SubscriptionError::NotUsingCW20Hook {})
            }
        }
        ExecuteMsg::CollectSubs { page_limit } => {
            commands::collect_subscriptions(deps, env, page_limit)
        }
        ExecuteMsg::ClaimCompensation {
            contributor,
            page_limit,
        } => commands::try_claim_contribution(add_on, deps, env, contributor, page_limit),
        ExecuteMsg::ClaimEmissions { os_id } => {
            commands::claim_subscriber_emissions(add_on, deps, env, os_id)
        }
        ExecuteMsg::UpdateContributor {
            contributor_addr,
            compensation,
        } => commands::update_contributor(deps, info, contributor_addr, compensation),
        ExecuteMsg::RemoveContributor { contributor_addr } => {
            commands::remove_contributor(deps, info, contributor_addr)
        }
        ExecuteMsg::UpdateSubscriptionConfig {
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
        ExecuteMsg::UpdateContributionConfig {
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token,
            emissions_amp_factor,
            emissions_offset,
            base_denom,
        } => commands::update_contribution_config(
            deps,
            env,
            info,
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token,
            emissions_amp_factor,
            emissions_offset,
            base_denom,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => SubscriptionAddOn::default().query(deps, env, message),
        // handle dapp-specific queries here
        QueryMsg::State {} => {
            let sub_state = SUB_STATE.load(deps.storage)?;
            let con_state = CON_STATE.load(deps.storage)?;
            to_binary(&StateResponse {
                contribution: con_state,
                subscription: sub_state,
            })
        }
        QueryMsg::Fee {} => {
            let config = SUB_CONFIG.load(deps.storage)?;
            to_binary(&SubscriptionFeeResponse {
                fee: Asset {
                    info: config.payment_asset,
                    amount: config.subscription_cost.into(),
                },
            })
        }
        QueryMsg::Config {} => {
            let sub_config = SUB_CONFIG.load(deps.storage)?;
            let con_config = CON_CONFIG.load(deps.storage)?;
            to_binary(&ConfigResponse {
                contribution: con_config,
                subscription: sub_config,
            })
        }
        QueryMsg::SubscriberState { os_id } => {
            let maybe_sub = CLIENTS.may_load(deps.storage, &os_id.to_be_bytes())?;
            let maybe_dormant_sub = DORMANT_CLIENTS.may_load(deps.storage, U32Key::new(os_id))?;
            let sub_state = if let Some(sub) = maybe_sub {
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
                return Err(StdError::generic_err("os is instance 0 or does not exist"));
            };
            Ok(sub_state)
        }
        QueryMsg::ContributorState { contributor_addr } => {
            let maybe_contributor =
                CONTRIBUTORS.may_load(deps.storage, contributor_addr.as_bytes())?;
            let sub_state = if let Some(compensation) = maybe_contributor {
                to_binary(&ContributorStateResponse { compensation })?
            } else {
                return Err(StdError::generic_err(
                    "provided address is not a contributor",
                ));
            };
            Ok(sub_state)
        }
    }
}
