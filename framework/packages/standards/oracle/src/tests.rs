use std::fmt::Debug;

use crate::msg::PriceResponse;
use cosmwasm_std::StdError;
use cw_orch::daemon::live_mock::mock_dependencies;
use cw_orch::prelude::*;

use crate::{OracleCommand, OracleError};

pub struct OracleCommandTester {
    chain: ChainInfoOwned,
    adapter: Box<dyn OracleCommand>,
}

pub fn expect_eq<T: PartialEq + Debug>(t1: T, t2: T) -> Result<(), StdError> {
    if t1 != t2 {
        Err(StdError::generic_err(format!(
            "Test failed, wrong result, expected : {:?}, got : {:?}",
            t1, t2
        )))?
    }
    Ok(())
}

impl OracleCommandTester {
    pub fn new<T: OracleCommand + 'static>(chain: ChainInfoOwned, module: T) -> Self {
        Self {
            chain,
            adapter: Box::new(module),
        }
    }

    pub fn test_query_price(&self, price_source_key: String) -> Result<PriceResponse, OracleError> {
        let deps = mock_dependencies(self.chain.clone());
        let ans_host = todo!();
        let price_response = self
            .adapter
            .price(deps.as_ref(), ans_host, price_source_key)?;
        Ok(price_response)
    }
}
