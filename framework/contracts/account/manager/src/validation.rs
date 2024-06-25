use abstract_sdk::std::PROXY;

use crate::error::ManagerError;

pub fn validate_not_proxy(module_id: &str) -> Result<(), ManagerError> {
    match module_id {
        PROXY => Err(ManagerError::CannotRemoveProxy {}),
        _ => Ok(()),
    }
}
