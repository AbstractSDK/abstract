use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use pandora_os::core::treasury::dapp_base::error::BaseDAppError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AnchorError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    BaseDAppError(#[from] BaseDAppError),
}
