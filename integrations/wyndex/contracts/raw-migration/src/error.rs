use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Tokens have to be migrated before stakers")]
    TokensNotMigrated,

    #[error("{0} isn't an authorized pool to withdraw into")]
    InvalidDestination(String),

    #[error("Method not implemented - only intended for migration")]
    NotImplemented,

    #[error("Cannot migrate contract type: `{0}`. Only works for wasmswap staking")]
    CannotMigrate(String),

    #[error("Got reply with unknown ID: {0}")]
    UnknownReply(u64),

    #[error("Target factory doesn't have unbonding period: {0}")]
    InvalidUnbondingPeriod(u64),

    #[error("Got reply with error, only handle success case")]
    ErrorReply,
}
