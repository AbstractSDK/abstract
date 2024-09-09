use abstract_sdk::std::ACCOUNT;

use crate::error::ManagerError;

pub fn validate_not_proxy(module_id: &str) -> Result<(), ManagerError> {
    match module_id {
        ACCOUNT => Err(ManagerError::CannotRemoveProxy {}),
        _ => Ok(()),
    }
}
