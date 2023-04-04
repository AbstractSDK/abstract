use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AnsHostError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("You must provide exactly two assets when adding liquidity")]
    NotTwoAssets {},

    #[error("{} is not part of the provided pool", id)]
    NotInPool { id: String },

    #[error("{} assets is not within range [{}-{}]", provided, min, max)]
    InvalidAssetCount {
        min: usize,
        max: usize,
        provided: usize,
    },

    #[error("Dex {} is not registered", dex)]
    UnregisteredDex { dex: String },

    #[error("Asset {} is not registered", asset)]
    UnregisteredAsset { asset: String },

    #[error("Dex {} is already registered", dex)]
    DexAlreadyRegistered { dex: String },
}
