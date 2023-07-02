use crate::error::ManagerError;
use abstract_sdk::core::PROXY;

pub fn validate_not_proxy(module_id: &str) -> Result<(), ManagerError> {
    match module_id {
        PROXY => Err(ManagerError::CannotRemoveProxy {}),
        _ => Ok(()),
    }
}
