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
pub struct TruncatedChainId(String);

impl TruncatedChainId {
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
        let parts = chain_id.rsplitn(2, '-');
        // the parts should look like [53159, cosmos-tesnet] or [cosmos-testnet], because we are using rsplitn
        Self(parts.last().unwrap().to_string())
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

impl std::fmt::Display for TruncatedChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TruncatedChainId {
    type Err = AbstractError;
    fn from_str(value: &str) -> AbstractResult<Self> {
        let chain_name = Self(value.to_string());
        chain_name.verify()?;
        Ok(chain_name)
    }
}

impl<'a> PrimaryKey<'a> for &TruncatedChainId {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl<'a> Prefixer<'a> for &TruncatedChainId {
    fn prefix(&self) -> Vec<Key> {
        self.0.prefix()
    }
}

impl KeyDeserialize for &TruncatedChainId {
    type Output = TruncatedChainId;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(TruncatedChainId(String::from_vec(value)?))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use cosmwasm_std::testing::mock_env;
    use speculoos::prelude::*;

    use super::*;

    #[coverage_helper::test]
    fn test_namespace() {
        let namespace = TruncatedChainId::new(&mock_env());
        assert_that!(namespace.as_str()).is_equal_to("cosmos-testnet");
    }

    #[coverage_helper::test]
    fn test_from_string() {
        let namespace = TruncatedChainId::from_string("test-me".to_string()).unwrap();
        assert_that!(namespace.as_str()).is_equal_to("test-me");
    }

    #[coverage_helper::test]
    fn test_from_str() {
        let namespace = TruncatedChainId::from_str("test-too").unwrap();
        assert_that!(namespace.as_str()).is_equal_to("test-too");
    }

    #[coverage_helper::test]
    fn test_to_string() {
        let namespace = TruncatedChainId::from_str("test").unwrap();
        assert_that!(namespace.to_string()).is_equal_to("test".to_string());
    }

    #[coverage_helper::test]
    fn test_from_str_long() {
        let namespace = TruncatedChainId::from_str("test-a-b-c-d-e-f").unwrap();
        assert_that!(namespace.as_str()).is_equal_to("test-a-b-c-d-e-f");
    }

    #[coverage_helper::test]
    fn string_key_works() {
        let k = &TruncatedChainId::from_str("test-abc").unwrap();
        let path = k.key();
        assert_eq!(1, path.len());
        assert_eq!(b"test-abc", path[0].as_ref());

        let joined = k.joined_key();
        assert_eq!(joined, b"test-abc")
    }

    // Failures

    #[coverage_helper::test]
    fn local_empty_fails() {
        TruncatedChainId::from_str("").unwrap_err();
    }

    #[coverage_helper::test]
    fn local_too_short_fails() {
        TruncatedChainId::from_str("a").unwrap_err();
    }

    #[coverage_helper::test]
    fn local_too_long_fails() {
        TruncatedChainId::from_str(&"a".repeat(MAX_CHAIN_NAME_LENGTH + 1)).unwrap_err();
    }

    #[coverage_helper::test]
    fn local_uppercase_fails() {
        TruncatedChainId::from_str("AAAAA").unwrap_err();
    }

    #[coverage_helper::test]
    fn local_non_alphanumeric_fails() {
        TruncatedChainId::from_str("a_aoeuoau").unwrap_err();
    }

    #[coverage_helper::test]
    fn from_chain_id() {
        let normal_chain_name = TruncatedChainId::from_chain_id("juno-1");
        assert_eq!(normal_chain_name, TruncatedChainId::_from_str("juno"));

        let postfixless_chain_name = TruncatedChainId::from_chain_id("juno");
        assert_eq!(postfixless_chain_name, TruncatedChainId::_from_str("juno"));
    }
}
