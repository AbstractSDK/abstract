use cosmwasm_std::{Coin, OverflowError, StdError, Uint128};
use thiserror::Error;

use cw_controllers::{AdminError, HookError};
use wynd_curve_utils::CurveError;
use wyndex::asset::AssetInfoValidated;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Hook(#[from] HookError),

    #[error("{0}")]
    Curve(#[from] CurveError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot rebond to the same unbonding period")]
    SameUnbondingRebond {},

    #[error("Rebond amount is invalid")]
    NoRebondAmount {},

    #[error("No claims that can be released currently")]
    NothingToClaim {},

    #[error(
        "Sender's CW20 token contract address {got} does not match one from config {expected}"
    )]
    Cw20AddressesNotMatch { got: String, expected: String },

    #[error("Trying to mass delegate {total} tokens, but only sent {amount_sent}.")]
    MassDelegateTooMuch {
        total: Uint128,
        amount_sent: Uint128,
    },

    #[error("No funds sent")]
    NoFunds {},

    #[error("No data in ReceiveMsg")]
    NoData {},

    #[error("No unbonding period found: {0}")]
    NoUnbondingPeriodFound(u64),

    #[error("No members to distribute tokens to")]
    NoMembersToDistributeTo {},

    #[error("There already is a distribution for {0}")]
    DistributionAlreadyExists(AssetInfoValidated),

    #[error("Cannot distribute the staked token")]
    InvalidAsset {},

    #[error("No distribution flow for this token: {0}")]
    NoDistributionFlow(Coin),

    #[error("Cannot add more than {0} distributions")]
    TooManyDistributions(u32),

    #[error("Cannot create new distribution after someone staked")]
    ExistingStakes {},

    #[error("Invalid distribution rewards")]
    InvalidRewards {},

    #[error("No reward duration provided for rewards distribution")]
    ZeroRewardDuration {},

    #[error("Cannot migrate stake without a converter contract")]
    NoConverter {},

    #[error("Fund distribution cannot start in the past.")]
    PastStartingTime {},

    #[error("Unbond all flag is already set to true")]
    FlagAlreadySet {},

    #[error("Cannot delegate when unbond all flag is set to true")]
    CannotDelegateIfUnbondAll {},

    #[error("Cannot distribute {what} when unbond all flag is set to true")]
    CannotDistributeIfUnbondAll { what: String },

    #[error("Cannot rebond when unbond all flag is set to true, unbond instead")]
    CannotRebondIfUnbondAll {},
}

impl From<OverflowError> for ContractError {
    fn from(e: OverflowError) -> Self {
        ContractError::Std(e.into())
    }
}
