use crate::handlers;
use crate::msg::{StakingExecuteMsg, StakingQueryMsg};
use crate::CW_STAKING;
use abstract_adapter::{export_endpoints, AdapterContract};
use abstract_staking_adapter_traits::CwStakingError;
use cosmwasm_std::{Empty, Response};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Staking contract adapter interface
pub type CwStakingAdapter =
    AdapterContract<CwStakingError, Empty, StakingExecuteMsg, StakingQueryMsg>;

/// Staking operation result
pub type StakingResult<T = Response> = Result<T, CwStakingError>;

/// Staking contract adapter
pub const CW_STAKING_ADAPTER: CwStakingAdapter =
    CwStakingAdapter::new(CW_STAKING, CONTRACT_VERSION, None)
        .with_execute(handlers::execute_handler)
        .with_query(handlers::query_handler);

// Export the endpoints for this contract
#[cfg(feature = "export")]
export_endpoints!(CW_STAKING_ADAPTER, CwStakingAdapter);
