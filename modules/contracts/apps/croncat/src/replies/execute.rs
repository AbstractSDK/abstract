use crate::{
    contract::{CroncatApp, CroncatResult},
    state::{ACTIVE_TASKS, REMOVED_TASK_MANAGER_ADDR, TEMP_TASK_KEY},
    utils::user_balance_nonempty,
};

use abstract_sdk::{
    features::{AbstractResponse, AccountIdentification},
    Execution,
};
use cosmwasm_std::{wasm_execute, CosmosMsg, DepsMut, Env, Reply};
use croncat_integration_utils::reply_handler::reply_handle_croncat_task_creation;
use croncat_sdk_manager::msg::ManagerExecuteMsg;

pub fn create_task_reply(deps: DepsMut, _env: Env, app: CroncatApp, reply: Reply) -> CroncatResult {
    let (task, bin) = reply_handle_croncat_task_creation(reply)?;
    let key = TEMP_TASK_KEY.load(deps.storage)?;
    ACTIVE_TASKS.save(deps.storage, key, &(task.task_hash.clone(), task.version))?;

    Ok(app
        .tag_response("create_task_reply")
        .add_attribute("task_hash", task.task_hash)
        .set_data(bin))
}

pub fn task_remove_reply(
    deps: DepsMut,
    _env: Env,
    app: CroncatApp,
    _reply: Reply,
) -> CroncatResult {
    let manager_addr = REMOVED_TASK_MANAGER_ADDR.load(deps.storage)?;
    let response = app.tag_response("task_remove_reply");
    let response = if user_balance_nonempty(
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
        response.add_message(executor_message)
    } else {
        response
    };
    Ok(response)
}
