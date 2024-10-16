use abstract_std::AbstractError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AnsHostError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Ownership(#[from] cw_ownable::OwnershipError),

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
}
