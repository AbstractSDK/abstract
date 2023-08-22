use crate::{error::AppError, handlers, msg::AppInstantiateMsg};
use abstract_app::AppContract;
use abstract_core::objects::dependency::StaticDependency;
use cosmwasm_std::{Empty, Response};
use croncat_app::{CRONCAT_ID, CRONCAT_MODULE_VERSION};

/// The version of your app
pub const ACC_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// the id of the app
pub const ACC_APP_ID: &str = "abstract:accountability";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type AccApp = AppContract<
    AppError,
    AppInstantiateMsg,
    AccountabilityExecuteMsg,
    AccountabilityQueryMsg,
    Empty,
>;

const ACC_APP: AccApp = AccApp::new(ACC_APP_ID, ACC_APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_dependencies(&[
        StaticDependency::new(CRONCAT_ID, &[CRONCAT_MODULE_VERSION]),
        StaticDependency::new(
            abstract_dex_adapter::EXCHANGE,
            &[abstract_dex_adapter::contract::CONTRACT_VERSION],
        ),
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(ACC_APP, AccApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(ACC_APP, AccApp, AccApp);
