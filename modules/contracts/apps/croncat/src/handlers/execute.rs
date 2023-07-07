use abstract_sdk::features::{AbstractNameService, AbstractResponse, AccountIdentification};
use abstract_sdk::{prelude::*, AccountAction};
use cosmwasm_std::{
    to_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, MessageInfo, ReplyOn, Response,
};
use croncat_integration_utils::task_creation::{get_croncat_contract, get_latest_croncat_contract};
use croncat_integration_utils::{MANAGER_NAME, TASKS_NAME};
use croncat_sdk_manager::msg::ManagerExecuteMsg;
use croncat_sdk_tasks::msg::{TasksExecuteMsg, TasksQueryMsg};
use croncat_sdk_tasks::types::{TaskRequest, TaskResponse};
use cw20::Cw20ExecuteMsg;
use cw_asset::AssetListUnchecked;

use crate::contract::{CroncatApp, CroncatResult};
use crate::error::AppError;
use crate::utils::{assert_module_installed, factory_addr, sort_funds, user_balance_nonempty};

use crate::msg::AppExecuteMsg;
use crate::replies::{TASK_CREATE_REPLY_ID, TASK_REMOVE_REPLY_ID};
use crate::state::{Config, ACTIVE_TASKS, CONFIG, REMOVED_TASK_MANAGER_ADDR, TEMP_TASK_KEY};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: CroncatApp,
    msg: AppExecuteMsg,
) -> CroncatResult {
    match msg {
        AppExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
        AppExecuteMsg::CreateTask {
            task,
            task_tag,
            assets,
        } => create_task(deps, env, info, app, task, task_tag, assets),
        AppExecuteMsg::RemoveTask { task_tag } => remove_task(deps, env, info, app, task_tag),
        AppExecuteMsg::RefillTask { task_tag, assets } => {
            refill_task(deps.as_ref(), env, info, app, task_tag, assets)
        }
        AppExecuteMsg::Purge { task_tags } => purge(deps, env, info, app, task_tags),
    }
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: CroncatApp) -> CroncatResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;

    CONFIG.save(deps.storage, &Config {})?;
    Ok(app.tag_response(Response::default(), "update_config"))
}

/// Create a task
fn create_task(
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    app: CroncatApp,
    task_request: Box<TaskRequest>,
    task_tag: String,
    assets: AssetListUnchecked,
) -> CroncatResult {
    if app
        .admin
        .assert_admin(deps.as_ref(), &msg_info.sender)
        .is_err()
    {
        assert_module_installed(deps.as_ref(), &msg_info.sender, &app)?;
    }
    let key = (msg_info.sender, task_tag);
    if ACTIVE_TASKS.has(deps.storage, key.clone()) {
        return Err(AppError::TaskAlreadyExists { task_tag: key.1 });
    }

    let (funds, cw20s) = sort_funds(deps.api, assets)?;

    let factory_addr = factory_addr(&deps.querier, &app.ans_host(deps.as_ref())?)?;
    let executor = app.executor(deps.as_ref());

    // Getting needed croncat addresses from factory
    let tasks_addr =
        get_latest_croncat_contract(&deps.querier, factory_addr.clone(), TASKS_NAME.to_owned())?;
    let manager_addr =
        get_latest_croncat_contract(&deps.querier, factory_addr, MANAGER_NAME.to_owned())?;

    // Making create task message that will be sended by the proxy
    let create_task_msg: CosmosMsg = wasm_execute(
        tasks_addr,
        &TasksExecuteMsg::CreateTask { task: task_request },
        funds,
    )?
    .into();
    let create_task_submessage = executor.execute_with_reply_and_data(
        create_task_msg,
        ReplyOn::Success,
        TASK_CREATE_REPLY_ID,
    )?;

    // Send any required cw20s before task creation
    let mut messages = vec![];
    for cw20 in cw20s {
        let cw20_transfer: CosmosMsg = wasm_execute(
            cw20.address,
            &Cw20ExecuteMsg::Send {
                contract: manager_addr.to_string(),
                amount: cw20.amount,
                msg: to_binary(&croncat_sdk_manager::msg::ManagerReceiveMsg::RefillTempBalance {})?,
            },
            vec![],
        )?
        .into();
        messages.push(executor.execute(vec![cw20_transfer.into()])?);
    }

    TEMP_TASK_KEY.save(deps.storage, &key)?;
    let response = Response::default()
        .add_messages(messages)
        .add_submessage(create_task_submessage);
    Ok(app.tag_response(response, "create_task"))
}

/// Remove a task
fn remove_task(
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    app: CroncatApp,
    task_tag: String,
) -> CroncatResult {
    if app
        .admin
        .assert_admin(deps.as_ref(), &msg_info.sender)
        .is_err()
    {
        assert_module_installed(deps.as_ref(), &msg_info.sender, &app)?;
    }
    let key = (msg_info.sender, task_tag);
    let (task_hash, task_version) = ACTIVE_TASKS.load(deps.storage, key.clone())?;

    let factory_addr = factory_addr(&deps.querier, &app.ans_host(deps.as_ref())?)?;
    let tasks_addr = get_croncat_contract(
        &deps.querier,
        factory_addr.clone(),
        TASKS_NAME.to_owned(),
        task_version.clone(),
    )?;
    let manager_addr = get_croncat_contract(
        &deps.querier,
        factory_addr,
        MANAGER_NAME.to_owned(),
        task_version,
    )?;

    ACTIVE_TASKS.remove(deps.storage, key);
    let task_response: TaskResponse = deps.querier.query_wasm_smart(
        tasks_addr.to_string(),
        &TasksQueryMsg::Task {
            task_hash: task_hash.to_owned(),
        },
    )?;

    // If there is still task by this hash on contract send remove message
    // If not - check if there is anything to withdraw and withdraw if needed
    let response = if task_response.task.is_some() {
        let remove_task_msg: CosmosMsg = wasm_execute(
            tasks_addr,
            &TasksExecuteMsg::RemoveTask { task_hash },
            vec![],
        )?
        .into();
        let executor_submessage = app.executor(deps.as_ref()).execute_with_reply(
            vec![remove_task_msg.into()],
            ReplyOn::Success,
            TASK_REMOVE_REPLY_ID,
        )?;
        REMOVED_TASK_MANAGER_ADDR.save(deps.storage, &manager_addr)?;
        Response::new().add_submessage(executor_submessage)
    } else if user_balance_nonempty(
        deps.as_ref(),
        app.proxy_address(deps.as_ref())?,
        manager_addr.clone(),
    )? {
        // withdraw locked balance
        let withdraw_msg: CosmosMsg = wasm_execute(
            manager_addr,
            &ManagerExecuteMsg::UserWithdraw { limit: None },
            vec![],
        )?
        .into();
        let executor_message = app
            .executor(deps.as_ref())
            .execute(vec![withdraw_msg.into()])?;
        Response::new().add_message(executor_message)
    } else {
        Response::new()
    };

    Ok(app.tag_response(response, "remove_task"))
}

/// Refill a task
fn refill_task(
    deps: Deps,
    _env: Env,
    msg_info: MessageInfo,
    app: CroncatApp,
    task_tag: String,
    assets: AssetListUnchecked,
) -> CroncatResult {
    if app.admin.assert_admin(deps, &msg_info.sender).is_err() {
        assert_module_installed(deps, &msg_info.sender, &app)?;
    }

    let key = (msg_info.sender, task_tag);
    let (task_hash, task_version) = ACTIVE_TASKS.load(deps.storage, key)?;

    let (funds, cw20s) = sort_funds(deps.api, assets)?;

    let executor = app.executor(deps);

    let factory_addr = factory_addr(&deps.querier, &app.ans_host(deps)?)?;
    let manager_addr = get_croncat_contract(
        &deps.querier,
        factory_addr,
        MANAGER_NAME.to_owned(),
        task_version,
    )?;

    let mut account_action: AccountAction = AccountAction::new();
    for cw20 in cw20s {
        let cw20_transfer: CosmosMsg = wasm_execute(
            cw20.address,
            &Cw20ExecuteMsg::Send {
                contract: manager_addr.to_string(),
                amount: cw20.amount,
                msg: to_binary(
                    &croncat_sdk_manager::msg::ManagerReceiveMsg::RefillTaskBalance {
                        task_hash: task_hash.clone(),
                    },
                )?,
            },
            vec![],
        )?
        .into();
        account_action.merge(cw20_transfer.into());
    }
    if !funds.is_empty() {
        let refill_task_msg: CosmosMsg = wasm_execute(
            manager_addr,
            &ManagerExecuteMsg::RefillTaskBalance { task_hash },
            funds,
        )?
        .into();
        account_action.merge(refill_task_msg.into());
    }
    let msg = executor.execute(vec![account_action])?;

    Ok(app.tag_response(Response::new().add_message(msg), "refill_task"))
}

fn purge(
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    app: CroncatApp,
    task_tags: Vec<String>,
) -> CroncatResult {
    // In case module got unregistered or admin got changed they have no reason to purge now
    if app
        .admin
        .assert_admin(deps.as_ref(), &msg_info.sender)
        .is_err()
    {
        assert_module_installed(deps.as_ref(), &msg_info.sender, &app)?;
    }

    for tag in task_tags {
        ACTIVE_TASKS.remove(deps.storage, (msg_info.sender.clone(), tag));
    }
    Ok(app.tag_response(Response::new(), "purge"))
}
