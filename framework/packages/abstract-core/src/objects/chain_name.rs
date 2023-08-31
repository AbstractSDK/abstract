use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Env, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::AbstractResult;

#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
/// The name of a chain, aka the chain-id without the post-fix number.
/// ex. `cosmoshub-4` -> `cosmoshub`, `juno-1` -> `juno`
pub struct ChainName(String);

impl ChainName {
    // Construct the chain name from the environment (chain-id)
    pub fn new(env: &Env) -> Self {
        let chain_id = &env.block.chain_id;
        // split on the last -
        // `cosmos-testnet-53159`
        // -> `cosmos-testnet` and `53159`
        let parts: Vec<&str> = chain_id.rsplitn(2, '-').collect();
        // the parts vector should look like [53159, cosmos-tesnet], because we are using rsplitn
        Self(parts[1].to_string())
    }

    /// check the formatting of the chain name
    pub fn check(&self) -> AbstractResult<()> {
        if self.0.contains('-') || !self.0.as_str().is_ascii() {
            return Err(crate::AbstractError::FormattingError {
                object: "chain_name".into(),
                expected: "chain_name-351".into(),
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
}

impl From<&str> for ChainName {
    /// unchecked conversion!
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ChainName {
    /// unchecked conversion!
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ToString for ChainName {
    fn to_string(&self) -> String {
        self.0.clone()
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
    use super::*;
    use cosmwasm_std::testing::mock_env;
    use speculoos::prelude::*;

    #[test]
    fn test_namespace() {
        let namespace = ChainName::new(&mock_env());
        assert_that!(namespace.as_str()).is_equal_to("cosmos-testnet");
    }

    #[test]
    fn test_from_string() {
        let namespace = ChainName::from("test".to_string());
        assert_that!(namespace.as_str()).is_equal_to("test");
    }

    #[test]
    fn test_from_str() {
        let namespace = ChainName::from("test");
        assert_that!(namespace.as_str()).is_equal_to("test");
    }

    #[test]
    fn test_to_string() {
        let namespace = ChainName::from("test");
        assert_that!(namespace.to_string()).is_equal_to("test".to_string());
    }

    #[test]
    fn string_key_works() {
        let k = &ChainName::from("test");
        let path = k.key();
        assert_eq!(1, path.len());
        assert_eq!(b"test", path[0].as_ref());

        let joined = k.joined_key();
        assert_eq!(joined, b"test")
    }
}
