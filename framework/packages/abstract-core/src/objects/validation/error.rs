use super::verifiers::DANGEROUS_CHARS;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ValidationError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("description too short, must be at least {0} characters")]
    DescriptionInvalidShort(usize),

    #[error("description too long, must be at most {0} characters")]
    DescriptionInvalidLong(usize),

    #[error(
        "description contains dangerous characters, including one of {:?}",
        DANGEROUS_CHARS
    )]
    DescriptionContainsDangerousCharacters {},

    #[error("link too short, must be at least {0} characters")]
    LinkInvalidShort(usize),

    #[error("link too long, must be at most {0} characters")]
    LinkInvalidLong(usize),

    #[error("link must start with http:// or https://")]
    LinkInvalidFormat {},

    #[error(
        "link contains dangerous characters, including one of {:?}",
        DANGEROUS_CHARS
    )]
    LinkContainsDangerousCharacters {},

    #[error("title/gov-type too short, must be at least {0} characters")]
    TitleInvalidShort(usize),

    #[error("title/gov-type too long, must be at most {0} characters")]
    TitleInvalidLong(usize),

    #[error(
        "title/gov-type contains dangerous characters, including one of {:?}",
        DANGEROUS_CHARS
    )]
    TitleContainsDangerousCharacters {},
}
