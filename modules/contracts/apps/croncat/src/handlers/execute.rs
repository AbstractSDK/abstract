use std::iter;

use abstract_app::sdk::{
    features::{AbstractNameService, AccountIdentification},
    prelude::*,
    AccountAction,
};
use cosmwasm_std::{
    to_json_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, MessageInfo, ReplyOn,
};
use croncat_integration_utils::{
    task_creation::{get_croncat_contract, get_latest_croncat_contract},
    MANAGER_NAME, TASKS_NAME,
};
use croncat_sdk_manager::msg::ManagerExecuteMsg;
use croncat_sdk_tasks::{
    msg::{TasksExecuteMsg, TasksQueryMsg},
    types::{TaskRequest, TaskResponse},
};
use cw20::Cw20ExecuteMsg;
use cw_asset::AssetListUnchecked;

use crate::{
    contract::{CroncatApp, CroncatResult},
    error::AppError,
    msg::AppExecuteMsg,
    replies::{TASK_CREATE_REPLY_ID, TASK_REMOVE_REPLY_ID},
    state::{Config, ACTIVE_TASKS, CONFIG, REMOVED_TASK_MANAGER_ADDR, TEMP_TASK_KEY},
    utils::{assert_module_installed, factory_addr, sort_funds, user_balance_nonempty},
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: CroncatApp,
    msg: AppExecuteMsg,
) -> CroncatResult {
    match msg {
        AppExecuteMsg::UpdateConfig {} => update_config(deps, env, info, module),
        AppExecuteMsg::CreateTask {
            task,
            task_tag,
            assets,
        } => create_task(deps, env, info, module, task, task_tag, assets),
        AppExecuteMsg::RemoveTask { task_tag } => remove_task(deps, env, info, module, task_tag),
        AppExecuteMsg::RefillTask { task_tag, assets } => {
            refill_task(deps.as_ref(), env, info, module, task_tag, assets)
        }
        AppExecuteMsg::Purge { task_tags } => purge(deps, env, info, module, task_tags),
    }
}

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: CroncatApp,
) -> CroncatResult {
    // Only the admin should be able to call this
    module
        .admin
        .assert_admin(deps.as_ref(), &env.contract.address, &msg_info.sender)?;

    CONFIG.save(deps.storage, &Config {})?;
    Ok(module.response("update_config"))
}

/// Create a task
fn create_task(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: CroncatApp,
    task_request: Box<TaskRequest>,
    task_tag: String,
    assets: AssetListUnchecked,
) -> CroncatResult {
    if module
        .admin
        .assert_admin(deps.as_ref(), &env.contract.address, &msg_info.sender)
        .is_err()
    {
        assert_module_installed(deps.as_ref(), &msg_info.sender, &module)?;
    }
    let key = (msg_info.sender, task_tag);
    if ACTIVE_TASKS.has(deps.storage, key.clone()) {
        return Err(AppError::TaskAlreadyExists { task_tag: key.1 });
    }

    let (funds, cw20s) = sort_funds(deps.api, assets)?;

    let name_service = module.name_service(deps.as_ref());
    let factory_addr = factory_addr(&name_service)?;
    let executor = module.executor(deps.as_ref());

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
                msg: to_json_binary(
                    &croncat_sdk_manager::msg::ManagerReceiveMsg::RefillTempBalance {},
                )?,
            },
            vec![],
        )?
        .into();
        messages.push(executor.execute(iter::once(cw20_transfer))?);
    }

    TEMP_TASK_KEY.save(deps.storage, &key)?;
    Ok(module
        .response("create_task")
        .add_messages(messages)
        .add_submessage(create_task_submessage))
}

/// Remove a task
fn remove_task(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: CroncatApp,
    task_tag: String,
) -> CroncatResult {
    if module
        .admin
        .assert_admin(deps.as_ref(), &env.contract.address, &msg_info.sender)
        .is_err()
    {
        assert_module_installed(deps.as_ref(), &msg_info.sender, &module)?;
    }
    let key = (msg_info.sender, task_tag);
    let (task_hash, task_version) = ACTIVE_TASKS.load(deps.storage, key.clone())?;

    let name_service = module.name_service(deps.as_ref());
    let factory_addr = factory_addr(&name_service)?;
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

    let response = module.response("remove_task");
    // If there is still task by this hash on contract send remove message
    // If not - check if there is anything to withdraw and withdraw if needed
    let response = if task_response.task.is_some() {
        let remove_task_msg: CosmosMsg = wasm_execute(
            tasks_addr,
            &TasksExecuteMsg::RemoveTask { task_hash },
            vec![],
        )?
        .into();
        let executor_submessage = module.executor(deps.as_ref()).execute_with_reply(
            iter::once(remove_task_msg),
            ReplyOn::Success,
            TASK_REMOVE_REPLY_ID,
        )?;
        REMOVED_TASK_MANAGER_ADDR.save(deps.storage, &manager_addr)?;
        response.add_submessage(executor_submessage)
    } else if user_balance_nonempty(
        deps.as_ref(),
        module.proxy_address(deps.as_ref())?,
        manager_addr.clone(),
    )? {
        // withdraw locked balance
        let withdraw_msg: CosmosMsg = wasm_execute(
            manager_addr,
            &ManagerExecuteMsg::UserWithdraw { limit: None },
            vec![],
        )?
        .into();
        let executor_message = module
            .executor(deps.as_ref())
            .execute(iter::once(withdraw_msg))?;
        response.add_message(executor_message)
    } else {
        response
    };

    Ok(response)
}

/// Refill a task
fn refill_task(
    deps: Deps,
    env: Env,
    msg_info: MessageInfo,
    module: CroncatApp,
    task_tag: String,
    assets: AssetListUnchecked,
) -> CroncatResult {
    if module
        .admin
        .assert_admin(deps, &env.contract.address, &msg_info.sender)
        .is_err()
    {
        assert_module_installed(deps, &msg_info.sender, &module)?;
    }

    let key = (msg_info.sender, task_tag);
    let (task_hash, task_version) = ACTIVE_TASKS.load(deps.storage, key)?;

    let (funds, cw20s) = sort_funds(deps.api, assets)?;

    let executor = module.executor(deps);

    let name_service = module.name_service(deps);
    let factory_addr = factory_addr(&name_service)?;
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
                msg: to_json_binary(
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

    Ok(module.response("refill_task").add_message(msg))
}

fn purge(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: CroncatApp,
    task_tags: Vec<String>,
) -> CroncatResult {
    // In case module got unregistered or admin got changed they have no reason to purge now
    if module
        .admin
        .assert_admin(deps.as_ref(), &env.contract.address, &msg_info.sender)
        .is_err()
    {
        assert_module_installed(deps.as_ref(), &msg_info.sender, &module)?;
    }

    for tag in task_tags {
        ACTIVE_TASKS.remove(deps.storage, (msg_info.sender.clone(), tag));
    }
    Ok(module.response("purge"))
}
