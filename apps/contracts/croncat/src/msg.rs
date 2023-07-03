use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;
use croncat_integration_utils::CronCatTaskRequest;
use cw_asset::AssetListUnchecked;

use crate::{contract::CroncatApp, state::Config};

// Expose the top-level app messages
abstract_app::app_messages!(CroncatApp, AppExecuteMsg, AppQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    UpdateConfig {},
    CreateTask {
        task: Box<CronCatTaskRequest>,
        task_tag: String,
        assets: AssetListUnchecked,
    },
    RemoveTask {
        task_tag: String,
    },
    RefillTask {
        task_tag: String,
        assets: AssetListUnchecked,
    },
    Purge {
        task_tags: Vec<String>,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum AppQueryMsg {
    /// Get config
    #[returns(ConfigResponse)]
    Config {},
    /// Get active tasks
    #[returns(ActiveTasksResponse)]
    ActiveTasks {
        #[cfg_attr(
            not(feature = "interface"),
            doc = "The addr and task tag to start listing after."
        )]
        start_after: Option<(String, String)>,
        #[cfg_attr(
            not(feature = "interface"),
            doc = "Maximum number of tasks to return. Default limit is 50, if not set"
        )]
        limit: Option<u32>,
        #[cfg_attr(
            not(feature = "interface"),
            doc = "On true check if this task exist on croncat contract and filter if it doesn't."
        )]
        #[cfg_attr(not(feature = "interface"), doc = "Defaults to false")]
        checked: Option<bool>,
    },
    /// Get active tasks by creator
    #[returns(ActiveTasksByCreatorResponse)]
    ActiveTasksByCreator {
        #[cfg_attr(not(feature = "interface"), doc = "The addr of creator of tasks")]
        creator_addr: String,
        #[cfg_attr(
            not(feature = "interface"),
            doc = "The task tag to start listing after."
        )]
        start_after: Option<String>,
        #[cfg_attr(
            not(feature = "interface"),
            doc = "Maximum number of tasks to return. Default limit is 50, if not set"
        )]
        limit: Option<u32>,
        #[cfg_attr(
            not(feature = "interface"),
            doc = "On true check if this task exist on croncat contract and filter if it doesn't."
        )]
        #[cfg_attr(not(feature = "interface"), doc = "Defaults to false")]
        checked: Option<bool>,
    },
    /// Get task info
    #[returns(croncat_sdk_tasks::types::TaskResponse)]
    TaskInfo {
        creator_addr: String,
        task_tag: String,
    },
    /// Get task balance
    #[returns(croncat_sdk_manager::types::TaskBalanceResponse)]
    TaskBalance {
        creator_addr: String,
        task_tag: String,
    },
    /// Get manager address of task
    /// Used to verify sender
    #[returns(Addr)]
    ManagerAddr {
        creator_addr: String,
        task_tag: String,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum Cw20HookMsg {
    Deposit {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cosmwasm_schema::cw_serde]
pub enum ActiveTasksResponse {
    Unchecked {
        tasks: Vec<(Addr, String)>,
    },
    Checked {
        scheduled_tasks: Vec<(Addr, String)>,
        removed_tasks: Vec<(Addr, String)>,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum ActiveTasksByCreatorResponse {
    Unchecked {
        tasks: Vec<String>,
    },
    Checked {
        scheduled_tasks: Vec<String>,
        removed_tasks: Vec<String>,
    },
}
