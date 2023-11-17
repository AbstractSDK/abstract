use cosmwasm_std::StdError;

use manager::error::ManagerError;
use proxy::error::ProxyError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AccountError {
    #[error("{0}")]
    Std(#[from] StdError),

    // #[error("{0}")]
    // Abstract(#[from] AbstractError),

    // #[error("{0}")]
    // AbstractSdk(#[from] AbstractSdkError),

    // #[error("{0}")]
    // Admin(#[from] AdminError),
    #[error(transparent)]
    Manager(#[from] ManagerError),

    #[error(transparent)]
    Proxy(#[from] ProxyError),
}
