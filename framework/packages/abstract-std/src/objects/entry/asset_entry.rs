use std::fmt::Display;

use cosmwasm_std::StdResult;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{constants::CHAIN_DELIMITER, AbstractError, AbstractResult};

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

impl PrimaryKey<'_> for AssetEntry {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl Prefixer<'_> for AssetEntry {
    fn prefix(&self) -> Vec<Key> {
        self.0.prefix()
    }
}

impl KeyDeserialize for AssetEntry {
    type Output = AssetEntry;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(AssetEntry(String::from_vec(value)?))
    }
}

impl KeyDeserialize for &AssetEntry {
    type Output = AssetEntry;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(AssetEntry(String::from_vec(value)?))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use rstest::rstest;

    use super::*;

    #[coverage_helper::test]
    fn test_asset_entry() {
        let mut entry = AssetEntry::new("CRAB");
        assert_eq!(entry.as_str(), "crab");
        entry.format();
        assert_eq!(entry.as_str(), "crab");
    }

    #[coverage_helper::test]
    fn test_src_chain() -> AbstractResult<()> {
        // technically invalid, but we don't care here
        let entry = AssetEntry::new("CRAB");
        assert_eq!(
            entry.src_chain(),
            Err(AbstractError::EntryFormattingError {
                actual: "crab".to_string(),
                expected: "src_chain>asset_name".to_string(),
            })
        );
        let entry = AssetEntry::new("osmosis>crab");
        assert_eq!(entry.src_chain(), Ok("osmosis".to_string()));
        let entry = AssetEntry::new("osmosis>juno>crab");
        assert_eq!(entry.src_chain(), Ok("osmosis".to_string()));

        Ok(())
    }

    #[rstest]
    #[case("CRAB")]
    #[case("")]
    #[case(">")]
    #[case("juno>")]
    fn test_src_chain_error(#[case] input: &str) {
        let entry = AssetEntry::new(input);

        assert_eq!(
            entry.src_chain(),
            Err(AbstractError::EntryFormattingError {
                actual: input.to_ascii_lowercase(),
                expected: "src_chain>asset_name".to_string(),
            })
        );
    }

    #[coverage_helper::test]
    fn test_from_string() {
        let entry = AssetEntry::from("CRAB".to_string());
        assert_eq!(entry.as_str(), "crab");
    }

    #[coverage_helper::test]
    fn test_from_str() {
        let entry = AssetEntry::from("CRAB");
        assert_eq!(entry.as_str(), "crab");
    }

    #[coverage_helper::test]
    fn test_from_ref_string() {
        let entry = AssetEntry::from(&"CRAB".to_string());
        assert_eq!(entry.as_str(), "crab");
    }

    #[coverage_helper::test]
    fn test_to_string() {
        let entry = AssetEntry::new("CRAB");
        assert_eq!(entry.to_string(), "crab".to_string());
    }

    #[coverage_helper::test]
    fn string_key_works() {
        let k = &AssetEntry::new("CRAB");
        let path = k.key();
        assert_eq!(1, path.len());
        assert_eq!(b"crab", path[0].as_ref());

        let joined = k.joined_key();
        assert_eq!(joined, b"crab")
    }
}
