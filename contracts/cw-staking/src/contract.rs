use crate::{error::StakingError, handlers};
use abstract_api::{export_endpoints, ApiContract};
use abstract_sdk::os::cw_staking::{CwStakingExecuteMsg, CwStakingQueryMsg, CW_STAKING};
use cosmwasm_std::{Empty, Response};

const MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type CwStakingApi = ApiContract<StakingError, Empty, CwStakingExecuteMsg, CwStakingQueryMsg>;
pub type CwStakingResult<T = Response> = Result<T, StakingError>;

pub const CW_STAKING_API: CwStakingApi = CwStakingApi::new(CW_STAKING, MODULE_VERSION, None)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

// Export the endpoints for this contract
#[cfg(not(feature = "library"))]
export_endpoints!(CW_STAKING_API, CwStakingApi);

#[cfg(all(feature = "phoenix-1", feature = "pisco-1"))]
compile_error!("feature \"phoenix-1\" and feature \"pisco-1\" cannot be enabled at the same time");
