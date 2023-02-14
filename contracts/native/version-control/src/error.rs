use abstract_os::objects::OsId;
use abstract_os::AbstractOsError;
use abstract_sdk::os::objects::module::ModuleInfo;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VCError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AbstractOs(#[from] AbstractOsError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Module {0} does not have a stored module reference")]
    ModuleNotFound(ModuleInfo),

    #[error("Module {0} cannot be updated")]
    NotUpdateableModule(ModuleInfo),

    #[error("OS ID {} is not in version control register", id)]
    MissingOsId { id: OsId },
}
impl From<cw_semver::Error> for VCError {
    fn from(err: cw_semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
