use core::result::Result::{Err, Ok};

use cosmwasm_std::{Decimal, StdError, StdResult};

use crate::contract::{
    MAX_DESC_LENGTH, MAX_LINK_LENGTH, MAX_QUORUM, MAX_THRESHOLD, MAX_TITLE_LENGTH, MIN_DESC_LENGTH,
    MIN_LINK_LENGTH, MIN_TITLE_LENGTH,
};
use crate::ContractError;

/**
 * Validates that the provided [Decimal] value is in between [0,max_value].
 */
pub(crate) fn validate_decimal_value(value: Decimal, max_value: Decimal) -> StdResult<()> {
    if value > max_value {
        Err(StdError::generic_err(""))
    } else {
        Ok(())
    }
}

/**
 * Validates the quorum parameter used to instantiate the contract. It should be between [0,1].
 */
pub fn validate_quorum(quorum: Decimal) -> Result<(), ContractError> {
    match validate_decimal_value(quorum, MAX_QUORUM) {
        Ok(_) => Ok(()),
        Err(_) => Err(ContractError::PollQuorumInvalidValue(
            MAX_QUORUM.to_string(),
        )),
    }
}

/**
 * Validates the threshold parameter used to instantiate the contract. It should be between [0,1].
 */
pub fn validate_threshold(threshold: Decimal) -> Result<(), ContractError> {
    match validate_decimal_value(threshold, MAX_THRESHOLD) {
        Ok(_) => Ok(()),
        Err(_) => Err(ContractError::PollThresholdInvalidValue(
            MAX_THRESHOLD.to_string(),
        )),
    }
}

/**
 * Validates that the link is valid when creating a poll.
 */
pub fn validate_poll_link(link: &Option<String>) -> Result<(), ContractError> {
    if let Some(link) = link {
        if link.len() < MIN_LINK_LENGTH {
            Err(ContractError::PollLinkInvalidShort(MIN_LINK_LENGTH))
        } else if link.len() > MAX_LINK_LENGTH {
            Err(ContractError::PollLinkInvalidLong(MAX_LINK_LENGTH))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/**
 * Validates that the title of the poll is valid, i.e. len() between [MIN_TITLE_LENGTH, MAX_TITLE_LENGTH].
 */
pub fn validate_poll_title(title: &str) -> Result<(), ContractError> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(ContractError::PollTitleInvalidShort(MIN_TITLE_LENGTH))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(ContractError::PollTitleInvalidLong(MAX_TITLE_LENGTH))
    } else {
        Ok(())
    }
}

/**
 * Validates that the description of the poll is valid, i.e. len() between [MIN_DESC_LENGTH, MAX_DESC_LENGTH].
 */
pub fn validate_poll_description(description: &str) -> Result<(), ContractError> {
    if description.len() < MIN_DESC_LENGTH {
        Err(ContractError::PollDescriptionInvalidShort(MIN_DESC_LENGTH))
    } else if description.len() > MAX_DESC_LENGTH {
        Err(ContractError::PollDescriptionInvalidLong(MAX_DESC_LENGTH))
    } else {
        Ok(())
    }
}
