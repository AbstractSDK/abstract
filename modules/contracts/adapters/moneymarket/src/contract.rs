use abstract_adapter::{export_endpoints, AdapterContract};
use abstract_moneymarket_standard::{
    msg::{MoneymarketExecuteMsg, MoneymarketInstantiateMsg, MoneymarketQueryMsg},
    MoneymarketError,
};
use cosmwasm_std::Response;

use crate::{handlers, MONEYMARKET_ADAPTER_ID};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type MoneymarketAdapter = AdapterContract<
    MoneymarketError,
    MoneymarketInstantiateMsg,
    MoneymarketExecuteMsg,
    MoneymarketQueryMsg,
>;
pub type MoneymarketResult<T = Response> = Result<T, MoneymarketError>;

pub const MONEYMARKET_ADAPTER: MoneymarketAdapter =
    MoneymarketAdapter::new(MONEYMARKET_ADAPTER_ID, CONTRACT_VERSION, None)
        .with_instantiate(handlers::instantiate_handler)
        .with_execute(handlers::execute_handler)
        .with_query(handlers::query_handler);

#[cfg(feature = "export")]
export_endpoints!(MONEYMARKET_ADAPTER, MoneymarketAdapter);
