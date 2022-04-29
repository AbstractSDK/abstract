#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_asset::Asset;
use cw_storage_plus::Map;
use pandora_os::registery::SUBSCRIPTION;
use protobuf::Message;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use semver::Version;

use pandora_os::modules::dapp_base::commands as dapp_base_commands;
use pandora_os::util::fee::Fee;

use pandora_os::modules::dapp_base::common::BaseDAppResult;
use pandora_os::modules::dapp_base::msg::BaseInstantiateMsg;
use pandora_os::modules::dapp_base::queries as dapp_base_queries;
use pandora_os::modules::dapp_base::state::{BaseState, ADMIN, BASESTATE};

use crate::error::SubscriptionError;
use crate::{commands, queries};
use pandora_os::modules::add_ons::subscription::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse, SubscriptionFeeResponse,
};
use pandora_os::modules::add_ons::subscription::state::*;
pub type SubscriptionResult = Result<Response, SubscriptionError>;

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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> SubscriptionResult {
    set_contract_version(deps.storage, SUBSCRIPTION, CONTRACT_VERSION)?;
    let base_state: BaseState = dapp_base_commands::handle_base_init(deps.as_ref(), msg.base)?;

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
    }.verify()?;

    let con_state: ContributionState = ContributionState {
        emissions_cap: Uint128::zero(),
        target: Uint64::zero(),
        expense: Uint64::zero(),
        total_weight: Uint128::zero(),
        emissions: Uint128::zero(),
        next_pay_day: Uint64::from(env.block.time.seconds() + MONTH),
    };

    SUB_CONFIG.save(deps.storage, &sub_config)?;
    SUB_STATE.save(deps.storage, &sub_state)?;
    CON_CONFIG.save(deps.storage, &con_config)?;
    CON_STATE.save(deps.storage, &con_state)?;
    BASESTATE.save(deps.storage, &base_state)?;

    CLIENTS.instantiate(deps.storage)?;
    CONTRIBUTORS.instantiate(deps.storage)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> SubscriptionResult {
    match msg {
        ExecuteMsg::Base(message) => {
            dapp_base_commands::handle_base_message(deps, info, message).map_err(|e| e.into())
        }
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::Pay { asset, os_id } => commands::try_pay(deps, info, asset, None, os_id),
        ExecuteMsg::CollectSubs { page_limit } => {
            commands::collect_subscriptions(deps, env, page_limit)
        }
        ExecuteMsg::ClaimCompensation {
            contributor,
            page_limit,
        } => commands::try_claim(deps, env, contributor, page_limit),
        ExecuteMsg::ClaimEmissions { os_id } => commands::claim_subscriber_emissions(deps, os_id),
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => dapp_base_queries::handle_base_query(deps, message),
        // handle dapp-specific queries here
        QueryMsg::State {} => {
            let state = SUB_STATE.load(deps.storage)?;
            to_binary(&Empty {})
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
            let config = SUB_CONFIG.load(deps.storage)?;
            to_binary(&SubscriptionFeeResponse {
                fee: Asset {
                    info: config.payment_asset,
                    amount: config.subscription_cost.into(),
                },
            })
        }
    }
}
