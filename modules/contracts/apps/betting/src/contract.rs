use crate::error::BetError;
use crate::handlers;
use crate::msg::{BetExecuteMsg, BetInstantiateMsg, BetQueryMsg};
use crate::ETF_APP_ID;
use abstract_app::AppContract;
use cosmwasm_std::{Empty, Response};
use cw20::Cw20ReceiveMsg;

pub(crate) const DEFAULT_LP_TOKEN_NAME: &str = "ETF LP token";
pub(crate) const DEFAULT_LP_TOKEN_SYMBOL: &str = "etfLP";

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub type BetResult<T = Response> = Result<T, BetError>;

pub type BetApp =
    AppContract<BetError, BetInstantiateMsg, BetExecuteMsg, BetQueryMsg, Empty, Cw20ReceiveMsg>;

const ETF_APP: BetApp = BetApp::new(ETF_APP_ID, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(ETF_APP, BetApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(ETF_APP, BetApp, BetApp);
