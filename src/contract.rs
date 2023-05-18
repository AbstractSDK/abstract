use crate::msg::TemplateMigrateMsg;
use crate::{
    dependencies::TEMPLATE_DEPS,
    error::TemplateError,
    handlers,
    msg::{TemplateExecuteMsg, TemplateInstantiateMsg, TemplateQueryMsg},
    replies::{self, INSTANTIATE_REPLY_ID},
    TEMPLATE_ID,
};
use abstract_app::AppContract;
use cosmwasm_std::Response;
use cw20::Cw20ReceiveMsg;

/// The version of your module to be uploaded
const MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The type of the result returned by your app's entrypoints.
pub type TemplateResult<T = Response> = Result<T, TemplateError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type TemplateApp = AppContract<
    TemplateError,
    TemplateInstantiateMsg,
    TemplateExecuteMsg,
    TemplateQueryMsg,
    TemplateMigrateMsg,
    Cw20ReceiveMsg,
>;

const TEMPLATE_APP: TemplateApp = TemplateApp::new(TEMPLATE_ID, MODULE_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_receive(handlers::receive_handler)
    .with_migrate(handlers::migrate_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)])
    .with_dependencies(TEMPLATE_DEPS);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(TEMPLATE_APP, TemplateApp);
