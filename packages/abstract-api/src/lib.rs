//! # Abstract api
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub type ApiResult<C = Empty> = Result<Response<C>, ApiError>;
// Default to Empty

pub use crate::state::ApiContract;
pub use error::ApiError;

pub mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
mod handler;
pub mod state;

#[cfg(feature = "schema")]
mod schema;

#[cfg(test)]
mod test_common {
    use crate::ApiError;
    use abstract_os::api;
    use abstract_sdk::AbstractSdkError;
    use cosmwasm_std::StdError;
    use thiserror::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error(transparent)]
        Api(#[from] ApiError),

        #[error("{0}")]
        Abstract(#[from] abstract_os::AbstractOsError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),
    }

    #[cosmwasm_schema::cw_serde]
    pub struct MockApiExecMsg;

    impl api::ApiExecuteMsg for MockApiExecMsg {}
}
