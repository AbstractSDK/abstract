use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DappError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Call is not a callback!")]
    NotCallback {},

    #[error("Not enough funds to perform trade")]
    Broke {},

    #[error("At least one trader must be configured")]
    TraderRequired {},

    #[error("The trader you wished to remove: {} was not present.", trader)]
    TraderNotPresent { trader: String },

    #[error("The trader you wished to add: {} is already present", trader)]
    TraderAlreadyPresent { trader: String },
}
