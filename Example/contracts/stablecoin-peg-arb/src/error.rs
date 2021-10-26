use thiserror::Error;

use cosmwasm_std::StdError;
use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum StableArbError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Call is not a callback!")]
    NotCallback {},

    #[error("The requested funds have the wrong denom")]
    WrongDenom {},

    #[error("Not enough funds to perform arb-trade")]
    Broke {},
}
