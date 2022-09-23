use abstract_os::objects::module::ModuleInfo;
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

    #[error("Module {0} does not have a stored code id")]
    MissingCodeId(ModuleInfo),

    #[error("Api {0} does not have a stored address")]
    MissingApi(ModuleInfo),

    #[error("Api {0} can not be updated")]
    ApiUpdate(ModuleInfo),

    #[error("Module {0} can not be updated")]
    CodeIdUpdate(ModuleInfo),

    #[error("OS ID {} is not in version control register", id)]
    MissingOsId { id: u32 },
}
impl From<semver::Error> for VCError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
