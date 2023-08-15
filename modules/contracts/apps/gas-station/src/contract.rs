use cosmwasm_std::{Empty, Response};

use abstract_app::AppContract;

use crate::msg::GasStationSudoMsg;
use crate::{
    error::AppError,
    handlers,
    msg::{AppInstantiateMsg, GasStationExecuteMsg, GasStationQueryMsg},
    replies,
};

/// The version of your app
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const GAS_STATION_APP_ID: &str = "abstract:gas-station";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type GasStationApp = AppContract<
    AppError,
    AppInstantiateMsg,
    GasStationExecuteMsg,
    GasStationQueryMsg,
    Empty,
    Empty,
    GasStationSudoMsg,
>;

const DCA_APP: GasStationApp = GasStationApp::new(GAS_STATION_APP_ID, VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_sudo(handlers::sudo_handler)
    .with_replies(&[(replies::CREATE_DENOM_REPLY_ID, replies::create_denom_reply)]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(DCA_APP, GasStationApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(DCA_APP, GasStationApp, GasStationApp);
