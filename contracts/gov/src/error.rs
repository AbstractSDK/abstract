use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Must deposit more than {0} token")]
    InsufficientProposalDeposit(u128),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("User has already voted")]
    AlreadyVoted {},

    #[error("Data should be given")]
    DataShouldBeGiven {},

    #[error("User does not have enough staked tokens")]
    InsufficientStaked {},

    #[error("Insufficient funds sent")]
    InsufficientFunds {},

    #[error("Nothing staked")]
    NothingStaked {},

    #[error("User is trying to withdraw too many tokens")]
    InvalidWithdrawAmount {},

    #[error("Cannot make a text proposal to expired state")]
    NoExecuteData {},

    #[error("Expire height has not been reached")]
    PollNotExpired {},

    #[error("Poll does not exist")]
    PollNotFound {},

    #[error("Poll is not in progress")]
    PollNotInProgress {},

    #[error("Poll is not in passed status")]
    PollNotPassed {},

    #[error("Voting period has not expired")]
    PollVotingPeriod {},

    #[error("Quorum must be between [0 and {0}]")]
    PollQuorumInvalidValue(String),

    #[error("Threshold must be between [0 and {0}]")]
    PollThresholdInvalidValue(String),

    #[error("Poll link too short, must be at least {0} characters")]
    PollLinkInvalidShort(usize),

    #[error("Poll link too long, must be at most {0} characters")]
    PollLinkInvalidLong(usize),

    #[error("Poll title too short, must be at least {0} characters")]
    PollTitleInvalidShort(usize),

    #[error("Poll title too long, must be at most {0} characters")]
    PollTitleInvalidLong(usize),

    #[error("Poll description too short, must be at least {0} characters")]
    PollDescriptionInvalidShort(usize),

    #[error("Poll description too long, must be at most {0} characters")]
    PollDescriptionInvalidLong(usize),

    #[error("Snapshot has already occurred")]
    SnapshotAlreadyOccurred {},

    #[error("Cannot snapshot at this height")]
    SnapshotHeight {},

    #[error("Timelock period has not expired")]
    TimelockNotExpired {},
}
