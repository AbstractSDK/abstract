use crate::state::{
    Compensation, ContributionState, ContributorsConfig, CACHED_CONTRIBUTION_STATE,
    CONTRIBUTION_CONFIG, CONTRIBUTION_STATE, CONTRIBUTORS,
};
use abstract_core::objects::AccountId;
use abstract_core::version_control::AccountBase;
use abstract_sdk::features::AbstractResponse;
use abstract_sdk::{
    AbstractSdkResult, AccountVerification, Execution, ModuleInterface, TransferInterface,
};
use abstract_subscription_interface::subscription::msg as subscr_msg;
use abstract_subscription_interface::subscription::state as subscr_state;

use abstract_subscription_interface::utils::suspend_os;
use abstract_subscription_interface::{ContributorsError, SUBSCRIPTION_ID};
use cosmwasm_std::{
    wasm_execute, Addr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage, SubMsg, Uint128, Uint64,
};
use cw_asset::{Asset, AssetInfoUnchecked};

use crate::contract::{App, AppResult};

use crate::msg::AppExecuteMsg;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::UpdateConfig {
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            emissions_amp_factor,
            emissions_offset,
            project_token_info,
        } => update_contribution_config(
            deps,
            env,
            info,
            app,
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token_info,
            emissions_amp_factor,
            emissions_offset,
        ),
        AppExecuteMsg::ClaimCompensation { os_id } => try_claim_compensation(app, deps, env, os_id),
        AppExecuteMsg::UpdateContributor {
            os_id,
            base_per_block,
            weight,
            expiration_block,
        } => update_contributor_compensation(
            deps,
            env,
            info,
            app,
            os_id,
            base_per_block,
            weight,
            expiration_block,
        ),
        AppExecuteMsg::RemoveContributor { os_id } => remove_contributor(deps, info, app, os_id),
    }
}

// #################### //
//      CONTRIBUTION    //
// #################### //

/// Function that adds/updates the contributor config of a given address
#[allow(clippy::too_many_arguments)]
pub fn update_contributor_compensation(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    app: App,
    contributor_os_id: AccountId,
    base_per_block: Option<Decimal>,
    weight: Option<u32>,
    expiration_block: Option<Uint64>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let _config = CONTRIBUTION_CONFIG.load(deps.storage)?;
    // Load all needed states
    let mut state = CONTRIBUTION_STATE.load(deps.storage)?;
    let contributor_addr = app
        .account_registry(deps.as_ref())
        .account_base(&contributor_os_id)?
        .manager;

    let maybe_compensation = CONTRIBUTORS.may_load(deps.storage, &contributor_addr)?;

    let new_compensation = match maybe_compensation {
        Some(current_compensation) => {
            // Can only update if already claimed last period.
            let sbuscription_addr = subscription_module_addr(&app, deps.as_ref())?;
            let twa_income = subscr_state::INCOME_TWA.query(&deps.querier, sbuscription_addr)?;
            if current_compensation.last_claim_block.u64() < twa_income.last_averaging_block_height
            {
                return try_claim_compensation(app, deps, env, contributor_os_id);
            };
            let compensation =
                current_compensation
                    .clone()
                    .overwrite(base_per_block, weight, expiration_block);
            if current_compensation.base_per_block > compensation.base_per_block {
                let (base_diff, weight_diff) = current_compensation.clone() - compensation.clone();
                state.total_weight = Uint128::from(
                    (state.total_weight.u128() as i128 - weight_diff as i128) as u128,
                );
                state.income_target -= base_diff;
            } else {
                let (base_diff, weight_diff) = compensation.clone() - current_compensation.clone();
                state.total_weight = Uint128::from(
                    (state.total_weight.u128() as i128 + weight_diff as i128) as u128,
                );
                state.income_target += base_diff;
            };
            Compensation {
                base_per_block: compensation.base_per_block,
                weight: compensation.weight,
                expiration_block: compensation.expiration_block,
                ..current_compensation
            }
        }
        None => {
            let compensation =
                Compensation::default().overwrite(base_per_block, weight, expiration_block);

            let os_id = app
                .account_registry(deps.as_ref())
                .assert_manager(&contributor_addr)
                .map_err(|_| ContributorsError::ContributorNotManager {})?;
            // TODO:

            // let subscriber = SUBSCRIBERS.load(deps.storage, &os_id)?;
            // if subscriber.manager_addr != contributor_addr {
            //     return Err(SubscriptionError::ContributorNotManager);
            // }
            // // New contributor doesn't pay for subscription but should be able to use os
            // let mut subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
            // subscription_state.active_subs -= 1;
            // SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
            // // Move to dormant. Prevents them from claiming user emissions
            // SUBSCRIBERS.remove(deps.storage, &os_id);
            // DORMANT_SUBSCRIBERS.save(deps.storage, &os_id, &subscriber)?;
            state.total_weight += Uint128::from(compensation.weight);
            state.income_target += compensation.base_per_block;
            Compensation {
                base_per_block: compensation.base_per_block,
                weight: compensation.weight,
                expiration_block: compensation.expiration_block,
                last_claim_block: env.block.height.into(),
            }
        }
    };

    CONTRIBUTORS.save(deps.storage, &contributor_addr, &new_compensation)?;
    CONTRIBUTION_STATE.save(deps.storage, &state)?;

    // Init vector for logging
    let attrs = vec![
        ("action", String::from("update_compensation")),
        ("for", contributor_addr.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs))
}

/// Removes the specified contributor
pub fn remove_contributor(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: App,
    os_id: AccountId,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let manager_address = app
        .account_registry(deps.as_ref())
        .account_base(&os_id)?
        .manager;
    remove_contributor_from_storage(deps.storage, manager_address.clone())?;
    // He must re-activate to join active set and earn emissions
    let msg = suspend_os(manager_address.clone(), true)?;
    // Init vector for logging
    let attrs = vec![
        ("action", String::from("remove_contributor")),
        ("address:", manager_address.to_string()),
    ];

    Ok(Response::new().add_message(msg).add_attributes(attrs))
}

// Check income
// Compute total contribution emissions
// Compute share of those emissions
// Compute share of income
/// Calculate the compensation for contribution
pub fn try_claim_compensation(app: App, deps: DepsMut, env: Env, os_id: AccountId) -> AppResult {
    let subscription_addr = subscription_module_addr(&app, deps.as_ref())?;
    let income_twa = subscr_state::INCOME_TWA.query(&deps.querier, subscription_addr.clone())?;
    if income_twa.need_refresh(&env) {
        // Refresh first
        Ok(Response::new().add_submessage(SubMsg::reply_on_success(
            wasm_execute(
                subscription_addr,
                &subscr_msg::ExecuteMsg::from(subscr_msg::SubscriptionExecuteMsg::RefreshTWA {}),
                vec![],
            )?,
            0,
        )))
    } else {
        let config = CONTRIBUTION_CONFIG.load(deps.storage)?;
        let mut state = CONTRIBUTION_STATE.load(deps.storage)?;
        // Update contribution state if income changes
        // TODO:
        let maybe_new_income = subscr_state::INCOME_TWA.try_update_value(&env, deps.storage)?;
        if let Some(income) = maybe_new_income {
            // Cache current state. This state will be used to pay out contributors of last period.
            CACHED_CONTRIBUTION_STATE.save(deps.storage, &state)?;
            // Overwrite current state with new income.
            update_contribution_state(deps.storage, env, &mut state, &config, income)?;
        }

        let cached_state = match CACHED_CONTRIBUTION_STATE.may_load(deps.storage)? {
            Some(state) => state,
            None => return Err(ContributorsError::AveragingPeriodNotPassed {}),
        };

        if cached_state.income_target.is_zero() {
            return Err(ContributorsError::TargetIsZero {});
        };

        let subscription_addr = subscription_module_addr(&app, deps.as_ref())?;
        let contributor_emissions = match subscr_state::SUBSCRIPTION_CONFIG
            .query(&deps.querier, subscription_addr.clone())?
            .subscription_per_block_emissions
        {
            subscr_state::EmissionType::IncomeBased(_) => {
                cached_state.emissions * (Decimal::one() - config.emission_user_share)
            }
            _ => cached_state.emissions,
        };

        let AccountBase {
            manager: contributor_address,
            proxy: contributor_proxy_address,
        } = app.account_registry(deps.as_ref()).account_base(&os_id)?;

        let mut compensation = CONTRIBUTORS.load(deps.storage, &contributor_address)?;
        let twa_data = subscr_state::INCOME_TWA.query(&deps.querier, subscription_addr.clone())?;

        if compensation.last_claim_block.u64() >= twa_data.last_averaging_block_height {
            // Already claimed previous period
            return Err(ContributorsError::CompensationAlreadyClaimed {});
        };

        let payable_blocks =
            if twa_data.last_averaging_block_height > compensation.expiration_block.u64() {
                // End of last period is after the expiration
                // Pay period between last claim and expiration
                remove_contributor_from_storage(deps.storage, contributor_address)?;
                compensation.expiration_block - compensation.last_claim_block
            } else {
                // pay full period
                let period = Uint64::from(twa_data.last_averaging_block_height)
                    - compensation.last_claim_block;
                // update compensation details
                compensation.last_claim_block = twa_data.last_averaging_block_height.into();
                CONTRIBUTORS.save(deps.storage, &contributor_address, &compensation)?;
                period
            };

        // Payout depends on how much income was earned over that period.
        let payout_ratio = cached_state.expense / cached_state.income_target;
        // Pay period between last claim and end of cached state.
        let base_amount: Uint128 =
            (compensation.base_per_block * payout_ratio) * Uint128::from(payable_blocks);
        // calculate token emissions
        let token_amount = if !cached_state.total_weight.is_zero() {
            contributor_emissions
                * Decimal::from_ratio(compensation.weight as u128, cached_state.total_weight)
        } else {
            Decimal::zero()
        };

        let sub_config = subscr_state::SUBSCRIPTION_CONFIG.query(&deps.querier, subscription_addr)?;
        let mut assets = vec![];
        // Construct msgs
        if !base_amount.is_zero() {
            let base_asset: Asset = Asset::new(sub_config.payment_asset, base_amount);
            assets.push(base_asset);
        }

        if !token_amount.is_zero() {
            let token_asset: Asset =
                Asset::new(config.token_info, token_amount * Uint128::from(1u32));
            assets.push(token_asset)
        }
        if assets.is_empty() {
            Err(ContributorsError::NoAssetsToSend {})
        } else {
            let bank = app.bank(deps.as_ref());
            let transfer_action = bank.transfer(assets, &contributor_proxy_address)?;
            Ok(Response::new()
                .add_message(app.executor(deps.as_ref()).execute(vec![transfer_action])?)
                .add_attribute("action", "claim_contribution"))
        }
    }
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

fn remove_contributor_from_storage(
    store: &mut dyn Storage,
    contributor_addr: Addr,
) -> StdResult<()> {
    // Load all needed states
    let mut state = CONTRIBUTION_STATE.load(store)?;

    let maybe_compensation = CONTRIBUTORS.may_load(store, &contributor_addr)?;

    match maybe_compensation {
        Some(current_compensation) => {
            state.total_weight -= Uint128::from(current_compensation.weight);
            state.income_target -= current_compensation.base_per_block;
            CONTRIBUTORS.remove(store, &contributor_addr);
            CONTRIBUTION_STATE.save(store, &state)?;
        }
        None => {
            return Err(StdError::GenericErr {
                msg: "contributor is not registered".to_string(),
            })
        }
    };
    Ok(())
}

// Only Admin can execute it
#[allow(clippy::too_many_arguments)]
pub fn update_contribution_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: App,
    protocol_income_share: Option<Decimal>,
    emission_user_share: Option<Decimal>,
    max_emissions_multiple: Option<Decimal>,
    token_info: Option<AssetInfoUnchecked>,
    emissions_amp_factor: Option<Uint128>,
    emissions_offset: Option<Uint128>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config = CONTRIBUTION_CONFIG.load(deps.storage)?;

    if let Some(protocol_income_share) = protocol_income_share {
        config.protocol_income_share = protocol_income_share;
    }

    if let Some(emission_user_share) = emission_user_share {
        config.emission_user_share = emission_user_share;
    }

    if let Some(max_emissions_multiple) = max_emissions_multiple {
        config.max_emissions_multiple = max_emissions_multiple;
    }

    if let Some(emissions_amp_factor) = emissions_amp_factor {
        config.emissions_amp_factor = emissions_amp_factor;
    }

    if let Some(token_info) = token_info {
        // validate address format
        config.token_info = token_info.check(deps.api, None)?;
    }

    if let Some(emissions_offset) = emissions_offset {
        // validate address format
        config.emissions_offset = emissions_offset;
    }

    CONTRIBUTION_CONFIG.save(deps.storage, &config.verify()?)?;

    Ok(Response::new().add_attribute("action", "update contribution config"))
}

fn subscription_module_addr(app: &App, deps: Deps) -> AbstractSdkResult<cosmwasm_std::Addr> {
    app.modules(deps).module_address(SUBSCRIPTION_ID)
}
