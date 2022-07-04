use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AddOnError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),
}
