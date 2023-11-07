use crate::error::ManagerError;
use abstract_sdk::framework::PROXY;

pub fn validate_not_proxy(module_id: &str) -> Result<(), ManagerError> {
    match module_id {
        PROXY => Err(ManagerError::CannotRemoveProxy {}),
        _ => Ok(()),
    }
}
