use cosmwasm_std::{Empty, Response};

use abstract_app::AppContract;

use crate::msg::JuryDutyInstantiateMsg;
use crate::{
    error::AppError,
    handlers,
    msg::{JuryDutyExecuteMsg, JuryDutyQueryMsg},
};

/// The version of your app
pub const JURY_DUTY_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const JURY_DUTY_APP_ID: &str = "test:jury-duty";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type JuryDutyApp =
    AppContract<AppError, JuryDutyInstantiateMsg, JuryDutyExecuteMsg, JuryDutyQueryMsg, Empty>;

const DICE_APP: JuryDutyApp = JuryDutyApp::new(JURY_DUTY_APP_ID, JURY_DUTY_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_nois_callback(handlers::nois_callback_handler);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(DICE_APP, JuryDutyApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(DICE_APP, JuryDutyApp, JuryDutyApp);
