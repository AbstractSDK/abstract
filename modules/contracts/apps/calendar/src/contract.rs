use abstract_app::AppContract;
use cosmwasm_std::Response;

use crate::{
    error::CalendarError,
    handlers,
    msg::{CalendarExecuteMsg, CalendarInstantiateMsg, CalendarMigrateMsg, CalendarQueryMsg},
};

/// The version of your app
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const APP_ID: &str = "abstract:calendar";

/// The type of the result returned by your app's entry points.
pub type CalendarAppResult<T = Response> = Result<T, CalendarError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type CalendarApp = AppContract<
    CalendarError,
    CalendarInstantiateMsg,
    CalendarExecuteMsg,
    CalendarQueryMsg,
    CalendarMigrateMsg,
>;

const APP: CalendarApp = CalendarApp::new(APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, CalendarApp);

abstract_app::cw_orch_interface!(APP, CalendarApp, CalendarAppInterface);
