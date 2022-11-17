use cosmwasm_std::StdError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ExtensionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Sender of request is not a Manager")]
    UnauthorizedExtensionRequest {},

    #[error("Sender of request is not a Manager or Trader")]
    UnauthorizedTraderExtensionRequest {},

    #[error("The trader you wished to remove: {} was not present.", trader)]
    TraderNotPresent { trader: String },

    #[error("The trader you wished to add: {} is already present", trader)]
    TraderAlreadyPresent { trader: String },

    #[error("This extension does not implement any custom queries")]
    NoCustomQueries,

    #[error("No IBC receive handler function provided")]
    MissingIbcReceiveHandler,
}
