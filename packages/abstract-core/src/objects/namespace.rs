use cosmwasm_std::StdResult;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::AbstractResult;

use super::module::validate_name;

/// Represents an Abstract namespace for modules
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Namespace(String);

impl Namespace {
    pub fn new(namespace: &str) -> Self {
        Self(namespace.to_owned())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Check that the namespace is valid
    pub fn validate(&self) -> AbstractResult<()> {
        validate_name(&self.0)?;
        Ok(())
    }
}

impl From<&str> for Namespace {
    fn from(namespace: &str) -> Self {
        Self::new(namespace)
    }
}

impl From<String> for Namespace {
    fn from(namespace: String) -> Self {
        Self::new(&namespace)
    }
}

impl From<&String> for Namespace {
    fn from(namespace: &String) -> Self {
        Self::new(namespace)
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> PrimaryKey<'a> for &Namespace {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl<'a> Prefixer<'a> for &Namespace {
    fn prefix(&self) -> Vec<Key> {
        self.0.prefix()
    }
}

impl KeyDeserialize for &Namespace {
    type Output = Namespace;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(Namespace(String::from_vec(value)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;

    #[test]
    fn test_namespace() {
        let namespace = Namespace::new("test");
        assert_that!(namespace.as_str()).is_equal_to("test");
    }

    #[test]
    fn test_from_string() {
        let namespace = Namespace::from("test".to_string());
        assert_that!(namespace.as_str()).is_equal_to("test");
    }

    #[test]
    fn test_from_str() {
        let namespace = Namespace::from("test");
        assert_that!(namespace.as_str()).is_equal_to("test");
    }

    #[test]
    fn test_from_ref_string() {
        let namespace = Namespace::from(&"test".to_string());
        assert_that!(namespace.as_str()).is_equal_to("test");
    }

    #[test]
    fn test_to_string() {
        let namespace = Namespace::new("test");
        assert_that!(namespace.to_string()).is_equal_to("test".to_string());
    }

    #[test]
    fn string_key_works() {
        let k = &Namespace::new("test");
        let path = k.key();
        assert_eq!(1, path.len());
        assert_eq!(b"test", path[0].as_ref());

        let joined = k.joined_key();
        assert_eq!(joined, b"test")
    }
}
