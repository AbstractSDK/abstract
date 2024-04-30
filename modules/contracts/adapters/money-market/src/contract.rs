use abstract_adapter::{export_endpoints, AdapterContract};
use abstract_money_market_standard::{
    msg::{MoneyMarketExecuteMsg, MoneyMarketInstantiateMsg, MoneyMarketQueryMsg},
    MoneyMarketError,
};
use cosmwasm_std::Response;

use crate::{handlers, MONEY_MARKET_ADAPTER_ID};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type MoneyMarketAdapter = AdapterContract<
    MoneyMarketError,
    MoneyMarketInstantiateMsg,
    MoneyMarketExecuteMsg,
    MoneyMarketQueryMsg,
>;
pub type MoneyMarketResult<T = Response> = Result<T, MoneyMarketError>;

pub const MONEY_MARKET_ADAPTER: MoneyMarketAdapter =
    MoneyMarketAdapter::new(MONEY_MARKET_ADAPTER_ID, CONTRACT_VERSION, None)
        .with_instantiate(handlers::instantiate_handler)
        .with_execute(handlers::execute_handler)
        .with_query(handlers::query_handler);

#[cfg(feature = "export")]
export_endpoints!(MONEY_MARKET_ADAPTER, MoneyMarketAdapter);

#[cfg(feature = "interface")]
abstract_adapter::cw_orch_interface!(
    MONEY_MARKET_ADAPTER,
    crate::contract::MoneyMarketAdapter,
    MoneyMarketInstantiateMsg,
    MoneyMarketAdapter
);
