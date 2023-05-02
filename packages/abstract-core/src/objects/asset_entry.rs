use crate::{AbstractError, AbstractResult};
use cosmwasm_std::StdResult;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub const CHAIN_DELIMITER: &str = ">";

/// An unchecked ANS asset entry. This is a string that is formatted as
/// `src_chain>[intermediate_chain>]asset_name`
#[derive(
    Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Default,
)]
pub struct AssetEntry(pub(crate) String);

impl AssetEntry {
    pub fn new(entry: &str) -> Self {
        Self(str::to_ascii_lowercase(entry))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn format(&mut self) {
        self.0 = self.0.to_ascii_lowercase();
    }

    /// Retrieve the source chain of the asset
    /// Example: osmosis>juno>crab returns osmosis
    /// Returns string to remain consistent with [`Self::asset_name`]
    pub fn src_chain(&self) -> AbstractResult<String> {
        let mut split = self.0.splitn(2, CHAIN_DELIMITER);

        match split.next() {
            Some(src_chain) => {
                if src_chain.is_empty() {
                    return self.entry_formatting_error();
                }
                // Ensure there's at least one more element (asset_name)
                let maybe_asset_name = split.next();
                if maybe_asset_name.is_some() && maybe_asset_name != Some("") {
                    Ok(src_chain.to_string())
                } else {
                    self.entry_formatting_error()
                }
            }
            None => self.entry_formatting_error(),
        }
    }

    fn entry_formatting_error(&self) -> AbstractResult<String> {
        Err(AbstractError::EntryFormattingError {
            actual: self.0.clone(),
            expected: "src_chain>asset_name".to_string(),
        })
    }
}

impl From<&str> for AssetEntry {
    fn from(entry: &str) -> Self {
        Self::new(entry)
    }
}

impl From<String> for AssetEntry {
    fn from(entry: String) -> Self {
        Self::new(&entry)
    }
}

impl From<&String> for AssetEntry {
    fn from(entry: &String) -> Self {
        Self::new(entry)
    }
}

impl Display for AssetEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> PrimaryKey<'a> for &AssetEntry {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    // TODO: make this key implementation use src_chain as prefix
    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl<'a> Prefixer<'a> for &AssetEntry {
    fn prefix(&self) -> Vec<Key> {
        self.0.prefix()
    }
}

impl KeyDeserialize for &AssetEntry {
    type Output = AssetEntry;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(AssetEntry(String::from_vec(value)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;
    use speculoos::prelude::*;

    #[test]
    fn test_asset_entry() {
        let mut entry = AssetEntry::new("CRAB");
        assert_that!(entry.as_str()).is_equal_to("crab");
        entry.format();
        assert_that!(entry.as_str()).is_equal_to("crab");
    }

    #[test]
    fn test_src_chain() -> AbstractResult<()> {
        // technically invalid, but we don't care here
        let entry = AssetEntry::new("CRAB");
        assert_that!(entry.src_chain())
            .is_err()
            .is_equal_to(AbstractError::EntryFormattingError {
                actual: "crab".to_string(),
                expected: "src_chain>asset_name".to_string(),
            });
        let entry = AssetEntry::new("osmosis>crab");
        assert_that!(entry.src_chain())
            .is_ok()
            .is_equal_to("osmosis".to_string());
        let entry = AssetEntry::new("osmosis>juno>crab");
        assert_that!(entry.src_chain())
            .is_ok()
            .is_equal_to("osmosis".to_string());

        Ok(())
    }

    #[rstest]
    #[case("CRAB")]
    #[case("")]
    #[case(">")]
    #[case("juno>")]
    fn test_src_chain_error(#[case] input: &str) {
        let entry = AssetEntry::new(input);

        assert_that!(entry.src_chain())
            .is_err()
            .is_equal_to(AbstractError::EntryFormattingError {
                actual: input.to_ascii_lowercase(),
                expected: "src_chain>asset_name".to_string(),
            });
    }

    #[test]
    fn test_from_string() {
        let entry = AssetEntry::from("CRAB".to_string());
        assert_that!(entry.as_str()).is_equal_to("crab");
    }

    #[test]
    fn test_from_str() {
        let entry = AssetEntry::from("CRAB");
        assert_that!(entry.as_str()).is_equal_to("crab");
    }

    #[test]
    fn test_from_ref_string() {
        let entry = AssetEntry::from(&"CRAB".to_string());
        assert_that!(entry.as_str()).is_equal_to("crab");
    }

    #[test]
    fn test_to_string() {
        let entry = AssetEntry::new("CRAB");
        assert_that!(entry.to_string()).is_equal_to("crab".to_string());
    }

    #[test]
    fn string_key_works() {
        let k = &AssetEntry::new("CRAB");
        let path = k.key();
        assert_eq!(1, path.len());
        assert_eq!(b"crab", path[0].as_ref());

        let joined = k.joined_key();
        assert_eq!(joined, b"crab")
    }
}
