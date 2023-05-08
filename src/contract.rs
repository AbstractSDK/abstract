use crate::{
    handlers,
    replies::{
        self,
        INSTANTIATE_REPLY_ID
    },
    error::TemplateError,
    msg::{TemplateExecuteMsg, TemplateInstantiateMsg, TemplateQueryMsg},
    TEMPLATE_ID,
    dependencies::TEMPLATE_DEPS,
};
use abstract_app::AppContract;
use cosmwasm_std::{Empty, Response};
use cw20::Cw20ReceiveMsg;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub type TemplateResult<T = Response> = Result<T, TemplateError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type TemplateApp = AppContract<
    TemplateError,
    TemplateInstantiateMsg,
    TemplateExecuteMsg,
    TemplateQueryMsg,
    Empty,
    Cw20ReceiveMsg,
>;

const TEMPLATE_APP: TemplateApp = TemplateApp::new(TEMPLATE_ID, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_receive(handlers::receive_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)])
    .with_dependencies(TEMPLATE_DEPS);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(TEMPLATE_APP, TemplateApp);
