use abstract_app::AppError as AbstractAppError;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

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
    Cw4GroupError(#[from] cw4_group::error::ContractError),

    #[error("{0}")]
    Cw3Error(#[from] cw3_fixed_multisig::ContractError),

    #[error("{0} is not a member of the group")]
    NotMember(String),

    #[error("Too many members, max is {0}")]
    TooManyMembers(u32),

    #[error("Threshold must be absolute count")]
    ThresholdMustBeAbsoluteCount,

    #[error("All members each must have one weight")]
    MembersMustHaveSameWeight,

    #[error("The jury has not been set for proposal {0}")]
    JuryNotSet(u64),

    #[error("Not jury member {0}")]
    NotJuryMember(String),
}
