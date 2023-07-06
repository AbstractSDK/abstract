#![allow(clippy::too_many_arguments)]

use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{
    wasm_execute, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
};
use cw_asset::{Asset, AssetList};

use crate::contract::{AppResult, DCAApp};

use crate::error::AppError;
use crate::msg::{DCAExecuteMsg, ExecuteMsg, Frequency};
use crate::state::{Config, DCAEntry, CONFIG, DCA_LIST, NEXT_ID};
use abstract_dex_adapter::api::DexInterface;
use abstract_sdk::AbstractSdkResult;
use croncat_app::croncat_integration_utils::{CronCatAction, CronCatTaskRequest};
use croncat_app::{CronCat, CronCatInterface};

/// Helper to for task creation message
fn create_convert_task_internal(
    env: Env,
    dca: DCAEntry,
    dca_id: String,
    cron_cat: CronCat<DCAApp>,
    config: Config,
) -> AbstractSdkResult<CosmosMsg> {
    let interval = dca.frequency.to_interval();
    let task = CronCatTaskRequest {
        interval,
        boundary: None,
        // TODO?: should it be argument?
        stop_on_fail: true,
        actions: vec![CronCatAction {
            msg: wasm_execute(
                env.contract.address,
                &ExecuteMsg::from(DCAExecuteMsg::Convert {
                    dca_id: dca_id.clone(),
                }),
                vec![],
            )?
            .into(),
            gas_limit: Some(300_000),
        }],
        queries: None,
        transforms: None,
        cw20: None,
    };
    let assets = AssetList::from(vec![Asset::native(
        config.native_denom,
        config.dca_creation_amount,
    )])
    .into();
    cron_cat.create_task(task, dca_id, assets)
}

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DCAApp,
    msg: DCAExecuteMsg,
) -> AppResult {
    match msg {
        DCAExecuteMsg::UpdateConfig {
            new_native_denom,
            new_dca_creation_amount,
            new_refill_threshold,
            new_max_spread,
        } => update_config(
            deps,
            info,
            app,
            new_native_denom,
            new_dca_creation_amount,
            new_refill_threshold,
            new_max_spread,
        ),
        DCAExecuteMsg::CreateDCA {
            source_asset,
            target_asset,
            frequency,
            dex,
        } => create_dca(
            deps,
            env,
            info,
            app,
            source_asset,
            target_asset,
            frequency,
            dex,
        ),
        DCAExecuteMsg::UpdateDCA {
            dca_id,
            new_source_asset,
            new_target_asset,
            new_frequency,
            new_dex,
        } => update_dca(
            deps,
            env,
            info,
            app,
            dca_id,
            new_source_asset,
            new_target_asset,
            new_frequency,
            new_dex,
        ),
        DCAExecuteMsg::CancelDCA { dca_id } => cancel_dca(deps, info, app, dca_id),
        DCAExecuteMsg::Convert { dca_id } => convert(deps, env, info, app, dca_id),
    }
}

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: DCAApp,
    new_native_denom: Option<String>,
    new_dca_creation_amount: Option<Uint128>,
    new_refill_threshold: Option<Uint128>,
    new_max_spread: Option<Decimal>,
) -> AppResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let old_config = CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            native_denom: new_native_denom.unwrap_or(old_config.native_denom),
            dca_creation_amount: new_dca_creation_amount.unwrap_or(old_config.dca_creation_amount),
            refill_threshold: new_refill_threshold.unwrap_or(old_config.refill_threshold),
            max_spread: new_max_spread.unwrap_or(old_config.max_spread),
        },
    )?;

    Ok(app.tag_response(Response::default(), "update_config"))
}

/// Create new DCA
fn create_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DCAApp,
    source_asset: OfferAsset,
    target_asset: AssetEntry,
    frequency: Frequency,
    dex_name: DexName,
) -> AppResult {
    // Only the admin should be able to create dca
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let config = CONFIG.load(deps.storage)?;

    // Simulate swap first
    app.dex(deps.as_ref(), dex_name.clone())
        .simulate_swap(source_asset.clone(), target_asset.clone())?;

    // Generate DCA ID
    let id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    let dca_id = format!("dca_{id}");

    let dca_entry = DCAEntry {
        source_asset,
        target_asset,
        frequency,
        dex: dex_name,
    };
    DCA_LIST.save(deps.storage, dca_id.clone(), &dca_entry)?;

    let cron_cat = app.cron_cat(deps.as_ref());
    let task_msg = create_convert_task_internal(env, dca_entry, dca_id.clone(), cron_cat, config)?;

    Ok(app.tag_response(
        Response::new()
            .add_message(task_msg)
            .add_attribute("dca_id", dca_id),
        "create_dca",
    ))
}

/// Update existing dca
fn update_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DCAApp,
    dca_id: String,
    new_source_asset: Option<OfferAsset>,
    new_target_asset: Option<AssetEntry>,
    new_frequency: Option<Frequency>,
    new_dex: Option<DexName>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Only if frequency is changed we have to re-create a task
    let recreate_task = new_frequency.is_some();

    let old_dca = DCA_LIST.load(deps.storage, dca_id.clone())?;
    let new_dca = DCAEntry {
        source_asset: new_source_asset.unwrap_or(old_dca.source_asset),
        target_asset: new_target_asset.unwrap_or(old_dca.target_asset),
        frequency: new_frequency.unwrap_or(old_dca.frequency),
        dex: new_dex.unwrap_or(old_dca.dex),
    };

    // Simulate swap for a new dca
    app.dex(deps.as_ref(), new_dca.dex.clone())
        .simulate_swap(new_dca.source_asset.clone(), new_dca.target_asset.clone())?;

    DCA_LIST.save(deps.storage, dca_id.clone(), &new_dca)?;

    let response = if recreate_task {
        let config = CONFIG.load(deps.storage)?;
        let cron_cat = app.cron_cat(deps.as_ref());
        let remove_task_msg = cron_cat.remove_task(dca_id.clone())?;
        let create_task_msg = create_convert_task_internal(env, new_dca, dca_id, cron_cat, config)?;
        Response::new().add_messages(vec![remove_task_msg, create_task_msg])
    } else {
        Response::new()
    };
    Ok(app.tag_response(response, "update_dca"))
}

/// Remove existing dca, remove task from cron_cat
fn cancel_dca(deps: DepsMut, info: MessageInfo, app: DCAApp, dca_id: String) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    DCA_LIST.remove(deps.storage, dca_id.clone());

    let cron_cat = app.cron_cat(deps.as_ref());
    let remove_task_msg = cron_cat.remove_task(dca_id)?;

    Ok(app.tag_response(Response::new().add_message(remove_task_msg), "cancel_dca"))
}

/// Execute swap if called my croncat manager
/// Refill task if needed
fn convert(deps: DepsMut, env: Env, info: MessageInfo, app: DCAApp, dca_id: String) -> AppResult {
    let config = CONFIG.load(deps.storage)?;
    let dca = DCA_LIST.load(deps.storage, dca_id.clone())?;

    let cron_cat = app.cron_cat(deps.as_ref());

    let manager_addr = cron_cat.query_manager_addr(env.contract.address.clone(), dca_id.clone())?;
    if manager_addr != info.sender {
        return Err(AppError::NotManagerConvert {});
    }
    let mut messages = vec![];

    // In case task running out of balance - refill it
    let task_balance = cron_cat
        .query_task_balance(env.contract.address, dca_id.clone())?
        .balance
        .unwrap();
    if task_balance.native_balance < config.refill_threshold {
        messages.push(
            cron_cat.refill_task(
                dca_id,
                AssetList::from(vec![Asset::native(
                    config.native_denom,
                    config.dca_creation_amount,
                )])
                .into(),
            )?,
        );
    }

    // TODO: remove dca on failed swap?
    // Or `stop_on_fail` should be enough
    messages.push(app.dex(deps.as_ref(), dca.dex).swap(
        dca.source_asset,
        dca.target_asset,
        Some(config.max_spread),
        None,
    )?);
    Ok(app.tag_response(Response::new().add_messages(messages), "convert"))
}
