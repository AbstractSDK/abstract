use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{
    wasm_execute, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
};
use cw_asset::{Asset, AssetList};

use crate::contract::{AppResult, ChallengeApp};

use crate::error::AppError;
use crate::msg::{AccExecuteMsg, ExecuteMsg, Frequency};
use crate::state::{AccEntry, ChallengeEntry, Config, CHALLENGE_LIST, CONFIG, NEXT_ID};
use abstract_dex_adapter::api::DexInterface;
use abstract_sdk::AbstractSdkResult;
use croncat_app::croncat_intergration_utils::{CronCatAction, CronCatTaskRequest};
use croncat_app::{CronCat, CronCatInterface};

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: ChallengeApp,
    new_native_denom: Option<String>,
    new_forfeit_amount: Option<Uint128>,
    new_refill_threshold: Option<Uint128>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let old_config = CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            native_denom: new_native_denom.unwrap_or(old_config.native_denom),
            forfeit_amount: new_forfeit_amount.unwrap_or(old_config.forfeit_amount),
        },
    )?;

    Ok(app.tag_response(Response::default(), "update_config"))
}

/// Create new Accountability
fn create_accountability(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    source_asset: OfferAsset,
    frequence: Frequency,
    dex_name: DexName,
) -> AppResult {
    // Only the admin should be able to create a challenge.
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let config = CONFIG.load(deps.storage)?;

    // Generate the challenge id
    let id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    let dca_id = format!("acc_{id}");

    let acc_entry = AccEntry {
        source_asset,
        frequency,
    };
    ACC_LIST.save(deps.storage, dca_id.clone(), &acc_entry)?;

    let cron_cat = app.cron_cat(deps.as_ref());
    //let task_msg =

    Ok(app.tag_response(
        Response::new()
            .add_message(task_msg)
            .add_attribute("acc_id", acc_id),
        "create_accountability",
    ))
}

/// Update an existing challenge  
fn update_accountability(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: ChallengeApp,
    acc_id: String,
    new_source_asset: Option<OfferAsset>,
    new_frequency: Option<Frequency>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Only if frequency is changed we have to re-create a task
    let recreate_task = new_frequency.is_some();
    let old_accountability = ACC_LIST.load(deps.storage, acc_id.clone())?;
    let new_accountability = ChallengeEntry {
        name: new_name.unwrap_or(old_accountability.name),
        source_asset: new_source_asset.unwrap_or(old_accountability.source_asset),
        frequency: new_frequency.unwrap_or(old_accountability.frequency),
    };

    DCA_LIST.save(deps.storage, acc_id.clone(), &new_accountability)?;

    let response = if recreate_task {
        let config = CONFIG.load(deps.storage)?;
        let cron_cat = app.cron_cat(deps.as_ref());
        let remove_task_msg = cron_cat.remove_task(acc_id.clone())?;
        // @TODO //let create_task_msg =
    };
}

fn cancel_accountability(
    deps: DepsMut,
    info: MessageInfo,
    app: ChallengeApp,
    acc_id: String,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    ACC_LIST.remove(deps.storage, acc_id.clone());

    let cron_cat = app.cron_cat(deps.as_ref());
    let remove_task_msg = cron_cat.remove_task(acc_id.clone())?;

    Ok(app.tag_response(
        Response::new().add_message(remove_task_msg),
        "cancel_accountability",
    ))
}

fn add_friend(
    deps: DepsMut,
    info: MessageInfo,
    app: AccApp,
    friend_address: String,
    balance: Uint128,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
}

fn create_task_internal(
    env: Env,
    dca: AccEntry,
    acc_id: String,
    cron_cat: CronCat<AccApp>,
    config: Config,
) -> AbstractSdkResult<CosmosMsg> {
}
