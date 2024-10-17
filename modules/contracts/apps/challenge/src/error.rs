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
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error(transparent)]
    Asset(#[from] AssetError),

    #[error(transparent)]
    Admin(#[from] AdminError),

    #[error(transparent)]
    DappError(#[from] AbstractAppError),

    #[error(transparent)]
    VoteError(#[from] VoteError),

    #[error(transparent)]
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
