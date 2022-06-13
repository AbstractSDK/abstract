use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VCError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error(
        "Version {} of module {} does not have a stored code id",
        version,
        module
    )]
    MissingCodeId { version: String, module: String },

    #[error("Version {} of Api {} does not have a stored address", version, module)]
    MissingApi { version: String, module: String },

    #[error("Version {} of Api {} can not be updated", version, module)]
    ApiUpdate { version: String, module: String },

    #[error("Version {} of module {} can not be updated", version, module)]
    CodeIdUpdate { version: String, module: String },

    #[error("OS ID {} is not in version control register", id)]
    MissingOsId { id: u32 },
}
impl From<semver::Error> for VCError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
