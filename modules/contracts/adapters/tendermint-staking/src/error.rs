use abstract_adapter::sdk::AbstractSdkError;
use abstract_adapter::std::AbstractError;
use abstract_adapter::AdapterError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TendermintStakeError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    AdapterError(#[from] AdapterError),
}
