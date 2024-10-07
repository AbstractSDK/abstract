use std::iter;

use abstract_app::sdk::{
    features::{AbstractResponse, AccountIdentification},
    Execution,
};
use cosmwasm_std::{
    from_json, wasm_execute, CosmosMsg, DepsMut, Env, Reply, StdError, SubMsgResponse, SubMsgResult,
};
use croncat_sdk_manager::msg::ManagerExecuteMsg;

use crate::{
    contract::{CroncatApp, CroncatResult},
    error::AppError,
    state::{ACTIVE_TASKS, REMOVED_TASK_MANAGER_ADDR, TEMP_TASK_KEY},
    utils::user_balance_nonempty,
};

pub fn create_task_reply(
    deps: DepsMut,
    _env: Env,
    module: CroncatApp,
    reply: Reply,
) -> CroncatResult {
    let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = reply.result else {
        return Err(AppError::Std(StdError::generic_err(
            "Failed to create a task",
        )));
    };

    let account_execution_response = cw_utils::parse_execute_response_data(&b)?
        .data
        .unwrap_or_default();
    let task_bin = cw_utils::parse_execute_response_data(&account_execution_response.0)?
        .data
        .unwrap_or_default();
    let task: croncat_integration_utils::CronCatTaskExecutionInfo = from_json(&task_bin)?;
    let key = TEMP_TASK_KEY.load(deps.storage)?;
    ACTIVE_TASKS.save(deps.storage, key, &(task.task_hash.clone(), task.version))?;

    Ok(module
        .response("create_task_reply")
        .add_attribute("task_hash", task.task_hash)
        .set_data(task_bin))
}

pub fn task_remove_reply(
    deps: DepsMut,
    _env: Env,
    module: CroncatApp,
    _reply: Reply,
) -> CroncatResult {
    let manager_addr = REMOVED_TASK_MANAGER_ADDR.load(deps.storage)?;
    let response = module.response("task_remove_reply");
    let response = if user_balance_nonempty(
        deps.as_ref(),
        module.account_address(deps.as_ref())?,
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
