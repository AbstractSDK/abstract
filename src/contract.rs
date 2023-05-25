use crate::msg::TemplateMigrateMsg;
use crate::{
    error::TemplateError,
    handlers,
    msg::{TemplateExecuteMsg, TemplateInstantiateMsg, TemplateQueryMsg},
    replies::{self, INSTANTIATE_REPLY_ID},
};
use abstract_app::AppContract;
use cosmwasm_std::Response;

/// The version of your app
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const APP_ID: &str = "yournamespace:template";

/// The type of the result returned by your app's entry points.
pub type TemplateResult<T = Response> = Result<T, TemplateError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type TemplateApp = AppContract<
    TemplateError,
    TemplateInstantiateMsg,
    TemplateExecuteMsg,
    TemplateQueryMsg,
    TemplateMigrateMsg,
>;

const TEMPLATE_APP: TemplateApp = TemplateApp::new(APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(TEMPLATE_APP, TemplateApp);
