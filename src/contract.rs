use crate::error::TemplateError;
use crate::handlers;
use crate::handlers::instantiate::INSTANTIATE_REPLY_ID;
use crate::msg::{TemplateExecuteMsg, TemplateInstantiateMsg, TemplateQueryMsg};
use crate::TEMPLATE_ID;
use abstract_app::AppContract;
use cosmwasm_std::{Empty, Response};
use cw20::Cw20ReceiveMsg;

pub(crate) const DEFAULT_LP_TOKEN_NAME: &str = "ETF LP token";
pub(crate) const DEFAULT_LP_TOKEN_SYMBOL: &str = "etfLP";

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub type TemplateResult<T = Response> = Result<T, TemplateError>;

pub type TemplateApp =
    AppContract<TemplateError, TemplateInstantiateMsg, TemplateExecuteMsg, TemplateQueryMsg, Empty, Cw20ReceiveMsg>;

const TEMPLATE_APP: TemplateApp = TemplateApp::new(TEMPLATE_ID, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_receive(handlers::receive_cw20)
    .with_replies(&[(INSTANTIATE_REPLY_ID, handlers::instantiate_reply)]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(TEMPLATE_APP, TemplateApp);
