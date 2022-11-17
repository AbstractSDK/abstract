use abstract_extension::ExtensionError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TendermintStakeError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ExtensionError(#[from] ExtensionError),
}
