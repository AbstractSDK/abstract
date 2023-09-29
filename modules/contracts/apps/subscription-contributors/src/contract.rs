use crate::{
    handlers,
    msg::{AppExecuteMsg, AppMigrateMsg, AppQueryMsg, ContributorsInstantiateMsg},
    replies::{self, INSTANTIATE_REPLY_ID},
};
use abstract_app::AppContract;
use abstract_core::objects::dependency::StaticDependency;
use abstract_subscription_interface::CONTRIBUTORS_ID;
use abstract_subscription_interface::{ContributorsError, SUBSCRIPTION_ID};
use cosmwasm_std::Response;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, ContributorsError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type App = AppContract<
    ContributorsError,
    ContributorsInstantiateMsg,
    AppExecuteMsg,
    AppQueryMsg,
    AppMigrateMsg,
>;

// Should be same versions
const SUBSCRIPTIONS_DEPENDENCY: StaticDependency =
    StaticDependency::new(SUBSCRIPTION_ID, &[APP_VERSION]);

const APP: App = App::new(CONTRIBUTORS_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)])
    .with_dependencies(&[SUBSCRIPTIONS_DEPENDENCY]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, App);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(APP, App, AppInterface);
