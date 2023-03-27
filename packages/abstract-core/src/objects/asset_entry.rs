use cosmwasm_std::StdResult;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub const CHAIN_DELIMITER: &str = ">";

/// May key to retrieve information on an asset
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
    pub fn src_chain(&self) -> String {
        self.0
            .split(CHAIN_DELIMITER)
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// Retrieve the asset name without the src chain
    /// Example: osmosis>juno>crab returns juno>crab
    pub fn asset_name(&self) -> String {
        self.0
            .split(CHAIN_DELIMITER)
            .skip(1)
            .collect::<Vec<&str>>()
            .join(CHAIN_DELIMITER)
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
    use speculoos::prelude::*;

    #[test]
    fn test_asset_entry() {
        let mut entry = AssetEntry::new("CRAB");
        assert_that!(entry.as_str()).is_equal_to("crab");
        entry.format();
        assert_that!(entry.as_str()).is_equal_to("crab");
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
