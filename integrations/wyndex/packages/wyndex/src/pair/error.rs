use crate::asset::MINIMUM_LIQUIDITY_AMOUNT;
use cosmwasm_std::{CheckedMultiplyRatioError, ConversionOverflowError, OverflowError, StdError};
use thiserror::Error;

/// This enum describes pair contract errors
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Decimal(#[from] cosmwasm_std::DecimalRangeExceeded),

    #[error("Unknown reply id '{0}'")]
    UnknownReply(u64),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Trading has not started yet")]
    TradingNotStarted {},

    #[error("The {0} address was set already and cannot be changed")]
    AddrAlreadySet(&'static str),

    #[error("Operation non supported")]
    NonSupported {},

    #[error("Event of zero transfer")]
    InvalidZeroAmount {},

    #[error("Operation exceeds max spread limit")]
    MaxSpreadAssertion {},

    #[error("Provided spread amount exceeds allowed limit")]
    AllowedSpreadAssertion {},

    #[error("Operation exceeds max splippage tolerance")]
    MaxSlippageAssertion {},

    #[error("Doubling assets in asset infos")]
    DoublingAssets {},

    #[error("Asset mismatch between the requested and the stored asset in contract")]
    AssetMismatch {},

    #[error("Pair is not migrated to the new admin!")]
    PairIsNotMigrated {},

    #[error("Initial liquidity must be more than {}", MINIMUM_LIQUIDITY_AMOUNT)]
    MinimumLiquidityAmountError {},

    #[error("Referral commission is higher than the allowed maximum")]
    ReferralCommissionTooHigh {},

    #[error("{0}")]
    CheckedMultiplyRatioError(#[from] CheckedMultiplyRatioError),

    #[error("Insufficient amount of liquidity")]
    LiquidityAmountTooSmall {},

    #[error("The target rate epoch is specified in seconds and has to be less than a week")]
    InvalidTargetRateEpoch {},

    #[error("A pool with a dynamic target rate can only have 2 assets, one native token and one cw20 token")]
    InvalidAssetsForTargetRate {},

    #[error("Amp coefficient must be greater than 0 and less than or equal to {max_amp}")]
    IncorrectAmp { max_amp: u64 },

    #[error(
        "The difference between the old and new amp value must not exceed {max_amp_change} times"
    )]
    MaxAmpChangeAssertion { max_amp_change: u64 },

    #[error(
        "Amp coefficient cannot be changed more often than once per {min_amp_changing_time} seconds"
    )]
    MinAmpChangingTimeAssertion { min_amp_changing_time: u64 },

    #[error("You need to provide init params")]
    InitParamsNotFound {},

    #[error("It is not possible to provide liquidity with one token for an empty pool")]
    InvalidProvideLPsWithSingleToken {},

    #[error("The asset {0} does not belong to the pair")]
    InvalidAsset(String),

    #[error("Fee bps in must be smaller than or equal to 10,000")]
    InvalidFeeBps {},

    #[error("Ask or offer asset is missed")]
    VariableAssetMissed {},

    #[error("Source and target assets are the same")]
    SameAssets {},

    #[error(
        "Invalid number of assets. This pair supports at least {min} and at most {max} assets within a pool"
    )]
    InvalidNumberOfAssets { min: usize, max: usize },

    #[error(
        "Invalid number of assets. Expected at least 2 and at most {max} assets, but got {provided}"
    )]
    TooManyAssets { max: usize, provided: usize },

    #[error("Contract has been frozen")]
    ContractFrozen {},

    #[error("Spot price parameters incorrect - max_trade must be bigger then 0")]
    SpotPriceInvalidMaxTrade {},

    #[error("Spot price parameters incorrect - target_price must be bigger then 0")]
    SpotPriceInvalidTargetPrice {},

    #[error("Spot price parameters incorrect - iterations must be bigger then 0 and less or equal then 100")]
    SpotPriceInvalidIterations {},
}

impl From<ContractError> for StdError {
    fn from(e: ContractError) -> Self {
        match e {
            ContractError::Std(e) => e,
            _ => StdError::generic_err(e.to_string()),
        }
    }
}

impl From<OverflowError> for ContractError {
    fn from(o: OverflowError) -> Self {
        StdError::from(o).into()
    }
}

impl From<ConversionOverflowError> for ContractError {
    fn from(o: ConversionOverflowError) -> Self {
        StdError::from(o).into()
    }
}
