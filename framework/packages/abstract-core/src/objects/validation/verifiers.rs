use super::ValidationError;
use core::result::Result::{Err, Ok};

pub(crate) const MIN_DESC_LENGTH: usize = 1;
pub(crate) const MAX_DESC_LENGTH: usize = 1024;
/// Minimum link length is 11, because the shortest url could be http://a.be
pub(crate) const MIN_LINK_LENGTH: usize = 11;
pub(crate) const MAX_LINK_LENGTH: usize = 128;
pub(crate) const MIN_TITLE_LENGTH: usize = 1;
pub(crate) const MAX_TITLE_LENGTH: usize = 64;

pub(crate) const DANGEROUS_CHARS: &[char] = &['"', '\'', '=', '>', '<'];

fn contains_dangerous_characters(input: &str) -> bool {
    input.chars().any(|c| DANGEROUS_CHARS.contains(&c))
}

fn is_valid_url(link: &str) -> bool {
    link.starts_with("http://") || link.starts_with("https://") || link.starts_with("ipfs://")
}

pub fn validate_link(link: &Option<String>) -> Result<(), ValidationError> {
    if let Some(link) = link {
        if link.len() < MIN_LINK_LENGTH {
            Err(ValidationError::LinkInvalidShort(MIN_LINK_LENGTH))
        } else if link.len() > MAX_LINK_LENGTH {
            Err(ValidationError::LinkInvalidLong(MAX_LINK_LENGTH))
        } else if !is_valid_url(link) {
            Err(ValidationError::LinkInvalidFormat {})
        } else if contains_dangerous_characters(link) {
            Err(ValidationError::LinkContainsDangerousCharacters {})
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

pub fn validate_name(title: &str) -> Result<(), ValidationError> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(ValidationError::TitleInvalidShort(MIN_TITLE_LENGTH))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(ValidationError::TitleInvalidLong(MAX_TITLE_LENGTH))
    } else if contains_dangerous_characters(title) {
        Err(ValidationError::TitleContainsDangerousCharacters {})
    } else {
        Ok(())
    }
}

pub fn validate_description(maybe_description: &Option<String>) -> Result<(), ValidationError> {
    if let Some(description) = maybe_description {
        if description.len() < MIN_DESC_LENGTH {
            return Err(ValidationError::DescriptionInvalidShort(MIN_DESC_LENGTH));
        } else if description.len() > MAX_DESC_LENGTH {
            return Err(ValidationError::DescriptionInvalidLong(MAX_DESC_LENGTH));
        } else if contains_dangerous_characters(description) {
            return Err(ValidationError::DescriptionContainsDangerousCharacters {});
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use speculoos::prelude::*;

    mod link {
        use super::*;

        #[rstest(
            input,
            case("https://www.google.com"),
            case("http://example.com"),
            case("https://example.net:8080")
        )]
        fn valid(input: &str) {
            assert_that!(validate_link(&Some(input.to_string()))).is_ok();
        }

        #[rstest(
            input,
            case("http://a.b"),
            case("://example.com"),
            case("example.com"),
            case("https://example.org/path?query=value"),
            case("https:/example.com")
        )]
        fn invalid(input: &str) {
            assert_that!(validate_link(&Some(input.to_string()))).is_err();
        }
    }

    mod name {
        use super::*;

        #[rstest(input,
        case("name"),
        case("name123"),
        case("name 123"),
        case("a"),
        case(& "a".repeat(MAX_TITLE_LENGTH)),
        case("name!$%&*+,-.;@^_`|~"),
        case("名前"),
        )]
        fn valid_names(input: &str) {
            assert_that!(validate_name(input)).is_ok();
        }

        #[rstest(input,
        case(""),
        case(& "a".repeat(MAX_TITLE_LENGTH + 1)),
        case("name<>'\""),
        )]
        fn invalid_names(input: &str) {
            assert_that!(validate_name(input)).is_err();
        }
    }

    mod description {
        use super::*;

        #[rstest(input,
        case("d"),
        case("description123"),
        case("description 123"),
        case(& "a".repeat(MAX_DESC_LENGTH)),
        case("description!$%&*+,-.;@^_`|~"),
        case("説明"),
        )]
        fn valid_descriptions(input: &str) {
            assert_that!(validate_description(&Some(input.to_string()))).is_ok();
        }

        #[rstest(input,
        case(""),
        case(& "a".repeat(MAX_DESC_LENGTH + 1)),
        case("description<>'\""),
        )]
        fn invalid_descriptions(input: &str) {
            assert_that!(validate_description(&Some(input.to_string()))).is_err();
        }
    }
}
