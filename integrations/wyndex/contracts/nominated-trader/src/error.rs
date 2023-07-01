use cosmwasm_std::StdError;
use thiserror::Error;
use wyndex::asset::AssetInfo;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized {info}")]
    Unauthorized { info: String },

    #[error("Cannot swap {0}. No swap destinations")]
    CannotSwap(AssetInfo),

    #[error("not enough pools to route assets to desired base token")]
    SwapError {},

    #[error("Invalid route {0} to {1}")]
    InvalidRoute(AssetInfo, AssetInfo),

    #[error("Invalid route. Pool {0} to {1} not found")]
    InvalidRouteNoPool(String, String),

    #[error("Invalid route destination. {0} cannot be swapped to desired base token")]
    InvalidRouteDestination(String),

    #[error("Max route length of {0} was reached")]
    MaxRouteDepth(u64),

    #[error("Cannot collect. Remove duplicate asset")]
    DuplicatedAsset {},
}
