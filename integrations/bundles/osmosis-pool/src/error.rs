use abstract_interface::AbstractInterfaceError;
use cosmwasm_std::StdError;
use cw_orch::prelude::CwOrchError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OsmosisPoolError {
    #[error(transparent)]
    Orch(#[from] CwOrchError),

    #[error(transparent)]
    AbstractInterface(#[from] AbstractInterfaceError),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OsmosisTestTube(#[from] osmosis_test_tube::RunnerError),
}
