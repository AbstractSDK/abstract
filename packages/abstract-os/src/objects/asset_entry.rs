use std::fmt::Display;

use cosmwasm_std::StdResult;

use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// May key to retrieve information on an asset
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
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

impl Display for AssetEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> PrimaryKey<'a> for AssetEntry {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl<'a> Prefixer<'a> for AssetEntry {
    fn prefix(&self) -> Vec<Key> {
        self.0.prefix()
    }
}

impl KeyDeserialize for AssetEntry {
    type Output = Self;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(Self(String::from_vec(value)?))
    }
}
