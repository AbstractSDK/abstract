use cosmwasm_std::StdError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ApiError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Sender of request is not a Manager")]
    UnauthorizedApiRequest {},

    #[error("Sender of request is not a Manager or Trader")]
    UnauthorizedTraderApiRequest {},

    #[error("The trader you wished to remove: {} was not present.", trader)]
    TraderNotPresent { trader: String },

    #[error("The trader you wished to add: {} is already present", trader)]
    TraderAlreadyPresent { trader: String },

    #[error("This API does not implement any custom queries")]
    NoCustomQueries,

    #[error("No target OS specified to execute on.")]
    NoTargetOS,
}
