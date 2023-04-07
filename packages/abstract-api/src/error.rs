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

    #[error("Sender: {sender} of request to {api} is not a Manager or Authorized Address")]
    UnauthorizedAddressApiRequest { api: String, sender: String },

    #[error("The authorized address to remove: {} was not present.", address)]
    AuthorizedAddressNotPresent { address: String },

    #[error("The authorized address to add: {} is already present", address)]
    AuthorizedAddressAlreadyPresent { address: String },

    #[error("Maximum authorized addresses ({}) reached", max)]
    TooManyAuthorizedAddresses { max: u32 },

    #[error("This api does not implement any custom queries")]
    NoCustomQueries,

    #[error("No IBC receive handler function provided")]
    MissingIbcReceiveHandler,
}
