use abstract_app::AppContract;
use cosmwasm_std::{Empty, Response};
use cw20::Cw20ReceiveMsg;

use crate::{
    error::EtfError,
    handlers,
    handlers::instantiate::INSTANTIATE_REPLY_ID,
    msg::{EtfExecuteMsg, EtfInstantiateMsg, EtfQueryMsg},
    ETF_APP_ID,
};

pub(crate) const DEFAULT_LP_TOKEN_NAME: &str = "ETF LP token";
pub(crate) const DEFAULT_LP_TOKEN_SYMBOL: &str = "etfLP";

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub type EtfResult<T = Response> = Result<T, EtfError>;

pub type EtfApp =
    AppContract<EtfError, EtfInstantiateMsg, EtfExecuteMsg, EtfQueryMsg, Empty, Cw20ReceiveMsg>;

const ETF_APP: EtfApp = EtfApp::new(ETF_APP_ID, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, handlers::instantiate_reply)]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(ETF_APP, EtfApp);

abstract_app::cw_orch_interface!(ETF_APP, EtfApp, Etf);
