use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error(transparent)]
    Admin(#[from] AdminError),
}
