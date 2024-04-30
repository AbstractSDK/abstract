use std::collections::HashMap;

use abstract_app::sdk::features::AbstractNameService;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, QuerierWrapper, StdResult};
use croncat_integration_utils::{task_creation::get_croncat_contract, MANAGER_NAME, TASKS_NAME};
use croncat_sdk_manager::{msg::ManagerQueryMsg, types::TaskBalanceResponse};
use croncat_sdk_tasks::{msg::TasksQueryMsg, types::TaskResponse};
use cw_storage_plus::Bound;

use crate::{
    contract::{CroncatApp, CroncatResult},
    msg::{ActiveTasksByCreatorResponse, ActiveTasksResponse, AppQueryMsg, ConfigResponse},
    state::{ACTIVE_TASKS, CONFIG},
    utils::factory_addr,
};

pub const DEFAULT_LIMIT: u32 = 50;

fn check_if_task_exists(
    querier: &QuerierWrapper,
    manager_addrs: &mut HashMap<String, Addr>,
    factory_addr: Addr,
    task_hash: String,
    task_version: String,
) -> bool {
    let manager_addr = if let Some(addr) = manager_addrs.get(&task_version) {
        addr.clone()
    } else {
        match get_croncat_contract(
            querier,
            factory_addr,
            MANAGER_NAME.to_owned(),
            task_version.clone(),
        ) {
            Ok(addr) => {
                manager_addrs.insert(task_version, addr.clone());
                addr
            }
            Err(_) => return false,
        }
    };
    matches!(
        croncat_manager::state::TASKS_BALANCES.query(querier, manager_addr, task_hash.as_bytes()),
        Ok(Some(_))
    )
}

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &CroncatApp,
    msg: AppQueryMsg,
) -> CroncatResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        AppQueryMsg::ActiveTasks {
            start_after,
            limit,
            checked,
        } => to_json_binary(&query_active_tasks(deps, app, start_after, limit, checked)?),
        AppQueryMsg::ActiveTasksByCreator {
            creator_addr,
            start_after,
            limit,
            checked,
        } => to_json_binary(&query_active_tasks_by_creator(
            deps,
            app,
            creator_addr,
            start_after,
            limit,
            checked,
        )?),
        AppQueryMsg::TaskInfo {
            creator_addr,
            task_tag,
        } => to_json_binary(&query_task_info(deps, app, creator_addr, task_tag)?),
        AppQueryMsg::TaskBalance {
            creator_addr,
            task_tag,
        } => to_json_binary(&query_task_balance(deps, app, creator_addr, task_tag)?),
        AppQueryMsg::ManagerAddr {
            creator_addr,
            task_tag,
        } => to_json_binary(&query_manager_addr(deps, app, creator_addr, task_tag)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

fn query_active_tasks(
    deps: Deps,
    app: &CroncatApp,
    start_after: Option<(String, String)>,
    limit: Option<u32>,
    checked: Option<bool>,
) -> CroncatResult<ActiveTasksResponse> {
    let check = checked.unwrap_or(false);
    let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;

    let start_after = match start_after {
        Some((addr, tag)) => Some((deps.api.addr_validate(&addr)?, tag)),
        None => None,
    };
    let iter = ACTIVE_TASKS.range(
        deps.storage,
        start_after.map(Bound::exclusive),
        None,
        cosmwasm_std::Order::Ascending,
    );

    let response = match check {
        true => {
            let name_service = app.name_service(deps);
            let factory_addr = factory_addr(&name_service)?;
            let mut manager_addrs = HashMap::new();
            let mut removed_tasks = Vec::new();

            // filter tasks that doesn't exist on croncat contract anymore
            let tasks_result: StdResult<Vec<(Addr, String)>> = iter
                .filter(|res| {
                    res.as_ref().map_or(true, |(k, (task_hash, version))| {
                        if check_if_task_exists(
                            &deps.querier,
                            &mut manager_addrs,
                            factory_addr.clone(),
                            task_hash.clone(),
                            version.clone(),
                        ) {
                            true
                        } else {
                            removed_tasks.push(k.clone());
                            false
                        }
                    })
                })
                .map(|res| res.map(|(k, _)| k))
                .take(limit)
                .collect();
            ActiveTasksResponse::Checked {
                scheduled_tasks: tasks_result?,
                removed_tasks,
            }
        }
        false => {
            let tasks_result: StdResult<Vec<(Addr, String)>> =
                iter.map(|res| res.map(|(k, _)| k)).take(limit).collect();
            ActiveTasksResponse::Unchecked {
                tasks: tasks_result?,
            }
        }
    };
    Ok(response)
}

fn query_active_tasks_by_creator(
    deps: Deps,
    app: &CroncatApp,
    creator: String,
    start_after: Option<String>,
    limit: Option<u32>,
    checked: Option<bool>,
) -> CroncatResult<ActiveTasksByCreatorResponse> {
    let addr = deps.api.addr_validate(&creator)?;
    let check = checked.unwrap_or(false);
    let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;

    let iter = ACTIVE_TASKS.prefix(addr).range(
        deps.storage,
        start_after.map(Bound::exclusive),
        None,
        cosmwasm_std::Order::Ascending,
    );

    match check {
        true => {
            let name_service = app.name_service(deps);
            let factory_addr = factory_addr(&name_service)?;
            let mut manager_addrs = HashMap::new();
            let mut removed_tasks = Vec::new();

            // filter tasks that doesn't exist on croncat contract anymore
            let tasks_res: StdResult<Vec<String>> = iter
                .filter(|res| {
                    res.as_ref().map_or(true, |(k, (task_hash, version))| {
                        if check_if_task_exists(
                            &deps.querier,
                            &mut manager_addrs,
                            factory_addr.clone(),
                            task_hash.clone(),
                            version.clone(),
                        ) {
                            true
                        } else {
                            removed_tasks.push(k.clone());
                            false
                        }
                    })
                })
                .map(|res| res.map(|(k, _)| k))
                .take(limit)
                .collect();
            Ok(ActiveTasksByCreatorResponse::Checked {
                scheduled_tasks: tasks_res?,
                removed_tasks,
            })
        }
        false => {
            let tasks_res: StdResult<Vec<String>> =
                iter.map(|res| res.map(|(k, _)| k)).take(limit).collect();
            Ok(ActiveTasksByCreatorResponse::Unchecked { tasks: tasks_res? })
        }
    }
}

fn query_task_info(
    deps: Deps,
    app: &CroncatApp,
    creator_addr: String,
    task_tag: String,
) -> CroncatResult<TaskResponse> {
    let creator_addr = deps.api.addr_validate(&creator_addr)?;
    let (task_hash, task_version) = ACTIVE_TASKS.load(deps.storage, (creator_addr, task_tag))?;

    let name_service = app.name_service(deps);
    let factory_addr = factory_addr(&name_service)?;
    let tasks_addr = get_croncat_contract(
        &deps.querier,
        factory_addr,
        TASKS_NAME.to_owned(),
        task_version,
    )?;

    let task_info: TaskResponse = deps
        .querier
        .query_wasm_smart(tasks_addr, &TasksQueryMsg::Task { task_hash })?;
    Ok(task_info)
}

fn query_task_balance(
    deps: Deps,
    app: &CroncatApp,
    creator_addr: String,
    task_tag: String,
) -> CroncatResult<TaskBalanceResponse> {
    let creator_addr = deps.api.addr_validate(&creator_addr)?;
    let (task_hash, task_version) = ACTIVE_TASKS.load(deps.storage, (creator_addr, task_tag))?;

    let name_service = app.name_service(deps);
    let factory_addr = factory_addr(&name_service)?;
    let manager_addr = get_croncat_contract(
        &deps.querier,
        factory_addr,
        MANAGER_NAME.to_owned(),
        task_version,
    )?;

    let task_balance: TaskBalanceResponse = deps
        .querier
        .query_wasm_smart(manager_addr, &ManagerQueryMsg::TaskBalance { task_hash })?;
    Ok(task_balance)
}

fn query_manager_addr(
    deps: Deps,
    app: &CroncatApp,
    creator_addr: String,
    task_tag: String,
) -> CroncatResult<Addr> {
    let creator_addr = deps.api.addr_validate(&creator_addr)?;
    let (_, task_version) = ACTIVE_TASKS.load(deps.storage, (creator_addr, task_tag))?;

    let name_service = app.name_service(deps);
    let factory_addr = factory_addr(&name_service)?;
    let manager_addr = get_croncat_contract(
        &deps.querier,
        factory_addr,
        MANAGER_NAME.to_owned(),
        task_version,
    )?;
    Ok(manager_addr)
}
