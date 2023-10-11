use crate::state::RoundId;
use abstract_app::AppError;
use abstract_core::objects::validation::ValidationError;
use abstract_core::objects::AccountId;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::{Addr, CheckedFromRatioError, OverflowError, StdError};
use cw_asset::{AssetError, AssetInfo, AssetInfoBase};
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum BetError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    Validation(#[from] ValidationError),

    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("Asset type: {0} is unsupported.")]
    UnsupportedAssetType(String),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    DappError(#[from] AppError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("The provided token: {} is not this vault's LP token", token)]
    NotLPToken { token: String },

    #[error("The asset you wished to remove: {} is not part of the vector", asset)]
    AssetNotPresent { asset: String },

    #[error("The asset you wished to add: {} is already part of the vector", asset)]
    AssetAlreadyPresent { asset: String },

    #[error("The provided token is not the base token")]
    WrongToken {},

    #[error("The provided native coin is not the same as the claimed deposit")]
    WrongNative {},

    #[error("It's required to use cw20 send message to add liquidity with cw20 tokens")]
    NotUsingCW20Hook {},

    #[error("The provided fee is invalid")]
    InvalidFee {},

    #[error("The provided bet is invalid")]
    InvalidBet {},

    #[error("The deposit asset {0} is not the base asset for holding value calculation")]
    DepositAssetNotBase(String),

    #[error("The actual amount of tokens transferred is different from the claimed amount.")]
    InvalidAmount {},

    #[error("Round {0} not found")]
    RoundNotFound(RoundId),

    // account not found
    #[error("Account {0} not found")]
    AccountNotFound(AccountId),

    #[error("Account {account_id} is not participating in {round_id}")]
    AccountNotParticipating {
        round_id: RoundId,
        account_id: AccountId,
    },
    #[error("Invalid asset. Expected: {expected}, Actual: {actual}")]
    InvalidAsset {
        expected: AssetInfo,
        actual: AssetInfoBase<Addr>,
    },

    #[error("Round {0} already closed")]
    RoundAlreadyClosed(RoundId),

    #[error("Round {0} not closed")]
    RoundNotClosed(RoundId),
}
