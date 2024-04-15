#![warn(missing_docs)]
//! # CronCat App
//!
//! `abstract::croncat`

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;
use croncat_integration_utils::CronCatTaskRequest;
use cw_asset::AssetListUnchecked;

use crate::{contract::CroncatApp, state::Config};

// Expose the top-level app messages
abstract_app::app_msg_types!(CroncatApp, AppExecuteMsg, AppQueryMsg);

/// Croncat instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {}

/// Croncat execute messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
#[impl_into(ExecuteMsg)]
pub enum AppExecuteMsg {
    /// Update config
    /// currently this method is just placeholder
    UpdateConfig {},
    /// Create new croncat task
    CreateTask {
        /// Task request
        task: Box<CronCatTaskRequest>,
        /// Custom task tag, which helps to use it after creation
        task_tag: String,
        /// List of assets to be attached to the task
        assets: AssetListUnchecked,
    },
    /// Remove task by tag and withdraw unused cw20 by this task
    RemoveTask {
        /// Task tag of tag which should get removed
        task_tag: String,
    },
    /// Refill a task
    RefillTask {
        /// Task tag of tag which should get refilled
        task_tag: String,
        /// List of assets to be added to the task balance
        assets: AssetListUnchecked,
    },
    /// Clean state
    /// Note: it will not check if task is active or not, use it mindfully
    Purge {
        /// List of task tags that should get cleaned from the state
        task_tags: Vec<String>,
    },
}

/// Croncat Query Messages
#[cosmwasm_schema::cw_serde]
#[impl_into(QueryMsg)]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum AppQueryMsg {
    /// Get config
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Get active tasks
    /// Returns [`ActiveTasksResponse`]
    #[returns(ActiveTasksResponse)]
    ActiveTasks {
        /// The addr and task tag to start listing after.
        start_after: Option<(String, String)>,
        /// Maximum number of tasks to return. Default limit is 50, if not set
        limit: Option<u32>,
        /// On true check if this task exist on croncat contract and filter if it doesn't.
        /// Defaults to false
        checked: Option<bool>,
    },
    /// Get active tasks by creator
    /// Returns [`ActiveTasksByCreatorResponse`]
    #[returns(ActiveTasksByCreatorResponse)]
    ActiveTasksByCreator {
        /// The addr of creator of tasks
        creator_addr: String,
        /// The task tag to start listing after.
        start_after: Option<String>,
        /// Maximum number of tasks to return. Default limit is 50, if not set
        limit: Option<u32>,
        /// On true check if this task exist on croncat contract and filter if it doesn't.
        /// Defaults to false
        checked: Option<bool>,
    },
    /// Get task info
    /// Returns [`croncat_sdk_tasks::types::TaskResponse`]
    #[returns(croncat_sdk_tasks::types::TaskResponse)]
    TaskInfo {
        /// The addr of creator of tasks
        creator_addr: String,
        /// The tag of the task
        task_tag: String,
    },
    /// Get task balance
    /// Returns [`croncat_sdk_manager::types::TaskBalanceResponse`]
    #[returns(croncat_sdk_manager::types::TaskBalanceResponse)]
    TaskBalance {
        /// The addr of creator of tasks
        creator_addr: String,
        /// The tag of the task
        task_tag: String,
    },
    /// Get croncat manager address of task
    /// Used to verify sender
    /// Returns [`Addr`]
    #[returns(Addr)]
    ManagerAddr {
        /// The addr of creator of tasks
        creator_addr: String,
        /// The tag of the task
        task_tag: String,
    },
}

#[cosmwasm_schema::cw_serde]
/// Response for config query
pub struct ConfigResponse {
    /// Config of the croncat module
    pub config: Config,
}

#[cosmwasm_schema::cw_serde]
/// Response for active_tasks query
pub enum ActiveTasksResponse {
    /// List of tasks in the module state, without checking if it's still active on croncat side
    Unchecked {
        /// Tasks in CronCat App state
        tasks: Vec<(Addr, String)>,
    },
    /// Lists of tasks in the module state, with checking if it's still active on croncat side
    Checked {
        /// tasks that are currently active
        scheduled_tasks: Vec<(Addr, String)>,
        /// tasks that got removed and can safely get purged
        removed_tasks: Vec<(Addr, String)>,
    },
}

#[cosmwasm_schema::cw_serde]
/// Response for active_tasks_by_creator query
pub enum ActiveTasksByCreatorResponse {
    /// List of tasks in the module state, without checking if it's still active on croncat side
    Unchecked {
        /// Tasks in CronCat App state
        tasks: Vec<String>,
    },
    /// Lists of tasks in the module state, with checking if it's still active on croncat side
    Checked {
        /// tasks that are currently active
        scheduled_tasks: Vec<String>,
        /// tasks that got removed and can safely get purged
        removed_tasks: Vec<String>,
    },
}
