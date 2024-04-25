use abstract_app::AppContract;
use cosmwasm_std::{Empty, Response};

use crate::{
    error::AppError,
    handlers,
    msg::{AppExecuteMsg, AppInstantiateMsg, AppQueryMsg},
    replies,
    replies::{TASK_CREATE_REPLY_ID, TASK_REMOVE_REPLY_ID},
};

/// The version of your app
pub const CRONCAT_MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const CRONCAT_ID: &str = "croncat:cron";

/// The type of the result returned by your app's entry points.
pub type CroncatResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type CroncatApp = AppContract<AppError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, Empty>;

pub const CRONCAT_APP: CroncatApp = CroncatApp::new(CRONCAT_ID, CRONCAT_MODULE_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_replies(&[
        (TASK_CREATE_REPLY_ID, replies::create_task_reply),
        (TASK_REMOVE_REPLY_ID, replies::task_remove_reply),
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(CRONCAT_APP, CroncatApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(CRONCAT_APP, CroncatApp, Croncat);

#[cfg(feature = "interface")]
impl<Chain: cw_orch::environment::CwEnv> abstract_app::abstract_interface::DependencyCreation
    for crate::Croncat<Chain>
{
    type DependenciesConfig = Empty;
}
