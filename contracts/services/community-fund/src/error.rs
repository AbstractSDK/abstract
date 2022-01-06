use thiserror::Error;

use cosmwasm_std::{StdError, Uint128};
use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum CommunityFundError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("There're not enough tokens in the fund, {0} > {1}.")]
    InsufficientFunds(Uint128, Uint128),
}
