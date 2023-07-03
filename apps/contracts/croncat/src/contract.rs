use crate::msg::AppMigrateMsg;
use crate::replies::{TASK_CREATE_REPLY_ID, TASK_REMOVE_REPLY_ID};
use crate::{
    error::AppError,
    handlers,
    msg::{AppExecuteMsg, AppInstantiateMsg, AppQueryMsg},
    replies::{self, INSTANTIATE_REPLY_ID},
};
use abstract_app::AppContract;
use cosmwasm_std::Response;

/// The version of your app
pub const CRONCAT_MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const CRONCAT_ID: &str = "croncat:app";

/// The type of the result returned by your app's entry points.
pub type CroncatResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type CroncatApp =
    AppContract<AppError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, AppMigrateMsg>;

pub const CRONCAT_APP: CroncatApp = CroncatApp::new(CRONCAT_ID, CRONCAT_MODULE_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_replies(&[
        (INSTANTIATE_REPLY_ID, replies::instantiate_reply),
        (TASK_CREATE_REPLY_ID, replies::create_task_reply),
        (TASK_REMOVE_REPLY_ID, replies::task_remove_reply),
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(CRONCAT_APP, CroncatApp);

#[cfg(feature = "interface")]
abstract_app::create_interface!(CRONCAT_APP, CroncatApp);
