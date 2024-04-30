use abstract_app::{
    sdk::AbstractSdkError,
    std::{
        objects::{validation::ValidationError, voting::VoteError},
        AbstractError,
    },
    AppError as AbstractAppError,
};
use cosmwasm_std::{StdError, Timestamp};
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

use crate::state::MAX_AMOUNT_OF_FRIENDS;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    DappError(#[from] AbstractAppError),

    #[error("{0}")]
    VoteError(#[from] VoteError),

    #[error("{0}")]
    ValidationError(#[from] ValidationError),

    #[error("Challenge not found")]
    ChallengeNotFound {},

    #[error("Voter not found")]
    VoterNotFound {},

    #[error("The challenge is not active for the action")]
    ChallengeNotActive {},

    #[error("The check in status is not correct for this action")]
    WrongCheckInStatus {},

    #[error("No friends found for the challenge")]
    ZeroFriends {},

    #[error("Friends limit reached, max: {MAX_AMOUNT_OF_FRIENDS}")]
    TooManyFriends {},

    #[error("Can't have duplicate friends addresses")]
    DuplicateFriends {},

    #[error("Can't edit friends during active proposal: {0}")]
    FriendsEditDuringProposal(Timestamp),

    #[error("Challenge expired")]
    ChallengeExpired {},

    #[error("Challenge has no proposals yet")]
    ExpectedProposal {},
}
