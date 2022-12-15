use cosmwasm_std::{Empty, Response};
use cw20::Cw20ReceiveMsg;

use abstract_app::{export_endpoints, AppContract};
use abstract_sdk::os::etf::{EtfExecuteMsg, EtfInstantiateMsg, EtfQueryMsg};
use abstract_sdk::os::ETF;

use crate::error::EtfError;
use crate::handlers::{self};

pub(crate) const DEFAULT_LP_TOKEN_NAME: &str = "ETF LP token";
pub(crate) const DEFAULT_LP_TOKEN_SYMBOL: &str = "etfLP";

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type EtfApp =
    AppContract<EtfError, EtfExecuteMsg, EtfInstantiateMsg, EtfQueryMsg, Empty, Cw20ReceiveMsg>;

pub type EtfResult = Result<Response, EtfError>;

const ETF_ADDON: EtfApp = EtfApp::new(ETF, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_receive(handlers::receive_cw20)
    .with_replies(&[(INSTANTIATE_REPLY_ID, handlers::instantiate_reply)]);

// Export handlers
#[cfg(not(feature = "library"))]
export_endpoints!(ETF_ADDON, EtfApp);
