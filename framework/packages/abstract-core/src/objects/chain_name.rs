use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Env, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::{AbstractError, AbstractResult};

pub const MAX_CHAIN_NAME_LENGTH: usize = 20;
pub const MIN_CHAIN_NAME_LENGTH: usize = 3;

#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
/// The name of a chain, aka the chain-id without the post-fix number.
/// ex. `cosmoshub-4` -> `cosmoshub`, `juno-1` -> `juno`
pub struct ChainName(String);

impl ChainName {
    // Construct the chain name from the environment (chain-id)
    pub fn new(env: &Env) -> Self {
        let chain_id = &env.block.chain_id;
        Self::from_chain_id(chain_id)
    }

    // Construct the chain name from the chain id
    pub fn from_chain_id(chain_id: &str) -> Self {
        // split on the last -
        // `cosmos-testnet-53159`
        // -> `cosmos-testnet` and `53159`
        let parts: Vec<&str> = chain_id.rsplitn(2, '-').collect();
        // the parts vector should look like [53159, cosmos-tesnet], because we are using rsplitn
        Self(parts[1].to_string())
    }

    pub fn from_string(value: String) -> AbstractResult<Self> {
        let chain_name = Self(value);
        chain_name.verify()?;
        Ok(chain_name)
    }

    /// verify the formatting of the chain name
    pub fn verify(&self) -> AbstractResult<()> {
        // check length
        if self.0.is_empty()
            || self.0.len() < MIN_CHAIN_NAME_LENGTH
            || self.0.len() > MAX_CHAIN_NAME_LENGTH
        {
            return Err(AbstractError::FormattingError {
                object: "chain-seq".into(),
                expected: format!("between {MIN_CHAIN_NAME_LENGTH} and {MAX_CHAIN_NAME_LENGTH}"),
                actual: self.0.len().to_string(),
            });
        // check character set
        } else if !self.0.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
            return Err(crate::AbstractError::FormattingError {
                object: "chain_name".into(),
                expected: "chain-name".into(),
                actual: self.0.clone(),
            });
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    // used for key implementation
    pub(crate) fn str_ref(&self) -> &String {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    /// Only use this if you are sure that the string is valid (e.g. from storage)
    pub(crate) fn _from_str(value: &str) -> Self {
        Self(value.to_string())
    }

    /// Only use this if you are sure that the string is valid (e.g. from storage)
    pub(crate) fn _from_string(value: String) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for ChainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ChainName {
    type Err = AbstractError;
    fn from_str(value: &str) -> AbstractResult<Self> {
        let chain_name = Self(value.to_string());
        chain_name.verify()?;
        Ok(chain_name)
    }
}

impl<'a> PrimaryKey<'a> for &ChainName {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl<'a> Prefixer<'a> for &ChainName {
    fn prefix(&self) -> Vec<Key> {
        self.0.prefix()
    }
}

impl KeyDeserialize for &ChainName {
    type Output = ChainName;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(ChainName(String::from_vec(value)?))
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::mock_env;
    use speculoos::prelude::*;

    use super::*;

    #[test]
    fn test_namespace() {
        let namespace = ChainName::new(&mock_env());
        assert_that!(namespace.as_str()).is_equal_to("cosmos-testnet");
    }

    #[test]
    fn test_from_string() {
        let namespace = ChainName::from_string("test-me".to_string()).unwrap();
        assert_that!(namespace.as_str()).is_equal_to("test-me");
    }

    #[test]
    fn test_from_str() {
        let namespace = ChainName::from_str("test-too").unwrap();
        assert_that!(namespace.as_str()).is_equal_to("test-too");
    }

    #[test]
    fn test_to_string() {
        let namespace = ChainName::from_str("test").unwrap();
        assert_that!(namespace.to_string()).is_equal_to("test".to_string());
    }

    #[test]
    fn test_from_str_long() {
        let namespace = ChainName::from_str("test-a-b-c-d-e-f").unwrap();
        assert_that!(namespace.as_str()).is_equal_to("test-a-b-c-d-e-f");
    }

    #[test]
    fn string_key_works() {
        let k = &ChainName::from_str("test-abc").unwrap();
        let path = k.key();
        assert_eq!(1, path.len());
        assert_eq!(b"test-abc", path[0].as_ref());

        let joined = k.joined_key();
        assert_eq!(joined, b"test-abc")
    }

    // Failures

    #[test]
    fn local_empty_fails() {
        ChainName::from_str("").unwrap_err();
    }

    #[test]
    fn local_too_short_fails() {
        ChainName::from_str("a").unwrap_err();
    }

    #[test]
    fn local_too_long_fails() {
        ChainName::from_str(&"a".repeat(MAX_CHAIN_NAME_LENGTH + 1)).unwrap_err();
    }

    #[test]
    fn local_uppercase_fails() {
        ChainName::from_str("AAAAA").unwrap_err();
    }

    #[test]
    fn local_non_alphanumeric_fails() {
        ChainName::from_str("a_aoeuoau").unwrap_err();
    }
}
