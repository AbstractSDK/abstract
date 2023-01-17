use crate::contract::{
    MAX_DESC_LENGTH, MAX_LINK_LENGTH, MAX_TITLE_LENGTH, MIN_DESC_LENGTH, MIN_LINK_LENGTH,
    MIN_TITLE_LENGTH,
};
use crate::error::ManagerError;
use abstract_os::PROXY;
use core::result::Result::{Err, Ok};

/**
 * Validates that the link is valid when creating a os.
 */
pub fn validate_link(link: &Option<String>) -> Result<(), ManagerError> {
    if let Some(link) = link {
        if link.len() < MIN_LINK_LENGTH {
            Err(ManagerError::LinkInvalidShort(MIN_LINK_LENGTH))
        } else if link.len() > MAX_LINK_LENGTH {
            Err(ManagerError::LinkInvalidLong(MAX_LINK_LENGTH))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/**
 * Validates that the title or gov type of the os is valid, i.e. len() between [MIN_TITLE_LENGTH, MAX_TITLE_LENGTH].
 */
pub fn validate_name_or_gov_type(title: &str) -> Result<(), ManagerError> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(ManagerError::TitleInvalidShort(MIN_TITLE_LENGTH))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(ManagerError::TitleInvalidLong(MAX_TITLE_LENGTH))
    } else {
        Ok(())
    }
}

/**
 * Validates that the description of the os is valid, i.e. len() between [MIN_DESC_LENGTH, MAX_DESC_LENGTH].
 */
pub fn validate_description(maybe_description: &Option<String>) -> Result<(), ManagerError> {
    if let Some(description) = maybe_description {
        if description.len() < MIN_DESC_LENGTH {
            return Err(ManagerError::DescriptionInvalidShort(MIN_DESC_LENGTH));
        } else if description.len() > MAX_DESC_LENGTH {
            return Err(ManagerError::DescriptionInvalidLong(MAX_DESC_LENGTH));
        }
    }
    Ok(())
}

pub fn validate_not_proxy(module_id: &str) -> Result<(), ManagerError> {
    match module_id {
        PROXY => Err(ManagerError::CannotRemoveProxy {}),
        _ => Ok(()),
    }
}
