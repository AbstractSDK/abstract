use thiserror::Error;

use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use white_whale::trader::TraderError;

#[derive(Error, Debug, PartialEq)]
pub enum TerraswapWrapperError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Trader(#[from] TraderError),
}
