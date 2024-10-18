use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AdapterError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("Sender: {sender} of request to {adapter} is not an Account or top-level owner")]
    UnauthorizedAdapterRequest { adapter: String, sender: String },

    #[error("Sender: {sender} of request to {adapter} is not an Account or Authorized Address")]
    UnauthorizedAddressAdapterRequest { adapter: String, sender: String },

    #[error(
        "The authorized address or module_id to remove: {} was not present.",
        addr_or_module_id
    )]
    AuthorizedAddressOrModuleIdNotPresent { addr_or_module_id: String },

    #[error(
        "The authorized address or module_id to add : {} was not valid.",
        addr_or_module_id
    )]
    AuthorizedAddressOrModuleIdNotValid { addr_or_module_id: String },

    #[error(
        "The authorized address or module_id to add: {} is already present",
        addr_or_module_id
    )]
    AuthorizedAddressOrModuleIdAlreadyPresent { addr_or_module_id: String },

    #[error("Maximum authorized addresses ({}) reached", max)]
    TooManyAuthorizedAddresses { max: u32 },

    #[error("This api does not implement any custom queries")]
    NoCustomQueries,

    #[error("No IBC receive handler function provided")]
    MissingIbcReceiveHandler,
}
