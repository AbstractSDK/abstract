use abstract_app::std::objects::dependency::StaticDependency;
use abstract_app::AppContract;
use cosmwasm_std::Response;

use crate::{
    error::AppError,
    handlers,
    msg::{AppExecuteMsg, AppInstantiateMsg, AppMigrateMsg, AppQueryMsg},
};

/// The version of your app
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const APP_ID: &str = "abstract:payment";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type PaymentApp =
    AppContract<AppError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, AppMigrateMsg>;

const DEX_DEPENDENCY: StaticDependency = StaticDependency::new(
    abstract_dex_adapter::DEX_ADAPTER_ID,
    &[abstract_dex_adapter::contract::CONTRACT_VERSION],
);

// ANCHOR: dependencies
const APP: PaymentApp = PaymentApp::new(APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    // Specify dependencies
    .with_dependencies(&[DEX_DEPENDENCY]);
// ANCHOR_END: dependencies

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, PaymentApp, crate::msg::CustomExecuteMsg);

abstract_app::cw_orch_interface!(
    APP,
    PaymentApp,
    PaymentAppInterface,
    crate::msg::CustomExecuteMsg
);
