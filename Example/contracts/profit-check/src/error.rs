use thiserror::Error;

use cosmwasm_std::StdError;
use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum ProfitCheckError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Cancel losing trade.")]
    CancelLosingTrade {},
}
