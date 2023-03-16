use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ApiError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("Sender: {sender} of request to {api} is not a Manager")]
    UnauthorizedApiRequest { api: String, sender: String },

    #[error("Sender: {sender} of request to {api} is not a Manager or Trader")]
    UnauthorizedTraderApiRequest { api: String, sender: String },

    #[error("The trader you wished to remove: {} was not present.", trader)]
    TraderNotPresent { trader: String },

    #[error("The trader you wished to add: {} is already present", trader)]
    TraderAlreadyPresent { trader: String },

    #[error("This api does not implement any custom queries")]
    NoCustomQueries,

    #[error("No IBC receive handler function provided")]
    MissingIbcReceiveHandler,
}
