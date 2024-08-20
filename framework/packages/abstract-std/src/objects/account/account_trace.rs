use std::fmt::Display;

use cosmwasm_std::{ensure, Env, StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::{constants::CHAIN_DELIMITER, objects::TruncatedChainId, AbstractError};

pub const MAX_TRACE_LENGTH: usize = 6;
pub(crate) const LOCAL: &str = "local";

/// The identifier of chain that triggered the account creation
#[cosmwasm_schema::cw_serde]
pub enum AccountTrace {
    Local,
    // path of the chains that triggered the account creation
    Remote(Vec<TruncatedChainId>),
}

impl KeyDeserialize for &AccountTrace {
    type Output = AccountTrace;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(AccountTrace::from_string(String::from_vec(value)?))
    }
}

impl<'a> PrimaryKey<'a> for AccountTrace {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        match self {
            AccountTrace::Local => LOCAL.key(),
            AccountTrace::Remote(chain_name) => {
                let len = chain_name.len();
                chain_name
                    .iter()
                    .enumerate()
                    .flat_map(|(s, c)| {
                        if s == len - 1 {
                            vec![c.str_ref().key()]
                        } else {
                            vec![c.str_ref().key(), CHAIN_DELIMITER.key()]
                        }
                    })
                    .flatten()
                    .collect::<Vec<Key>>()
            }
        }
    }
}

impl KeyDeserialize for AccountTrace {
    type Output = AccountTrace;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(AccountTrace::from_string(String::from_vec(value)?))
    }
}

impl<'a> Prefixer<'a> for AccountTrace {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

impl AccountTrace {
    /// verify the formatting of the Account trace chain
    pub fn verify(&self) -> Result<(), AbstractError> {
        match self {
            AccountTrace::Local => Ok(()),
            AccountTrace::Remote(chain_trace) => {
                // Ensure the trace length is limited
                ensure!(
                    chain_trace.len() <= MAX_TRACE_LENGTH,
                    AbstractError::FormattingError {
                        object: "chain-seq".into(),
                        expected: format!("between 1 and {MAX_TRACE_LENGTH}"),
                        actual: chain_trace.len().to_string(),
                    }
                );
                for chain in chain_trace {
                    chain.verify()?;
                    if chain.as_str().eq(LOCAL) {
                        return Err(AbstractError::FormattingError {
                            object: "chain-seq".into(),
                            expected: "not 'local'".into(),
                            actual: chain.to_string(),
                        });
                    }
                }
                Ok(())
            }
        }
    }

    /// assert that the account trace is a remote account and verify the formatting
    pub fn verify_remote(&self) -> Result<(), AbstractError> {
        if &Self::Local == self {
            Err(AbstractError::Std(StdError::generic_err(
                "expected remote account trace",
            )))
        } else {
            self.verify()
        }
    }

    /// assert that the trace is local
    pub fn verify_local(&self) -> Result<(), AbstractError> {
        if let &Self::Remote(..) = self {
            return Err(AbstractError::Std(StdError::generic_err(
                "expected local account trace",
            )));
        }
        Ok(())
    }

    /// push the `env.block.chain_name` to the chain trace
    pub fn push_local_chain(&mut self, env: &Env) {
        match &self {
            AccountTrace::Local => {
                *self = AccountTrace::Remote(vec![TruncatedChainId::new(env)]);
            }
            AccountTrace::Remote(path) => {
                let mut path = path.clone();
                path.push(TruncatedChainId::new(env));
                *self = AccountTrace::Remote(path);
            }
        }
    }

    /// push a chain name to the account's path
    pub fn push_chain(&mut self, chain_name: TruncatedChainId) {
        match &self {
            AccountTrace::Local => {
                *self = AccountTrace::Remote(vec![chain_name]);
            }
            AccountTrace::Remote(path) => {
                let mut path = path.clone();
                path.push(chain_name);
                *self = AccountTrace::Remote(path);
            }
        }
    }

    /// **No verification is done here**
    ///
    /// **only use this for deserialization**
    pub(crate) fn from_string(trace: String) -> Self {
        let acc = if trace == LOCAL {
            Self::Local
        } else {
            Self::Remote(
                trace
                    .split(CHAIN_DELIMITER)
                    .map(TruncatedChainId::_from_str)
                    .collect(),
            )
        };
        acc
    }

    /// **No verification is done here**
    ///
    /// **only use this for deserialization**
    #[allow(unused)]
    pub(crate) fn from_str(trace: &str) -> Result<Self, AbstractError> {
        let acc = if trace == LOCAL {
            Self::Local
        } else {
            Self::Remote(
                trace
                    .split(CHAIN_DELIMITER)
                    .map(TruncatedChainId::_from_str)
                    .collect(),
            )
        };
        acc.verify()?;
        Ok(acc)
    }
}

impl TryFrom<&str> for AccountTrace {
    type Error = AbstractError;

    fn try_from(trace: &str) -> Result<Self, Self::Error> {
        if trace == LOCAL {
            Ok(Self::Local)
        } else {
            let chain_trace: Vec<TruncatedChainId> = trace
                .split(CHAIN_DELIMITER)
                .map(|t| TruncatedChainId::from_string(t.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Self::Remote(chain_trace))
        }
    }
}

impl Display for AccountTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountTrace::Local => write!(f, "{}", LOCAL),
            AccountTrace::Remote(chain_name) => write!(
                f,
                "{}",
                // "juno>terra>osmosis"
                chain_name
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<&str>>()
                    .join(CHAIN_DELIMITER)
            ),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use std::str::FromStr;

    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    use super::*;

    mod format {
        use super::*;
        use crate::objects::truncated_chain_id::MAX_CHAIN_NAME_LENGTH;

        #[test]
        fn local_works() {
            let trace = AccountTrace::from_str(LOCAL).unwrap();
            assert_eq!(trace, AccountTrace::Local);
        }

        #[test]
        fn remote_works() {
            let trace = AccountTrace::from_str("bitcoin").unwrap();
            assert_eq!(
                trace,
                AccountTrace::Remote(vec![TruncatedChainId::from_str("bitcoin").unwrap()])
            );
        }

        #[test]
        fn remote_multi_works() {
            let trace = AccountTrace::from_str("bitcoin>ethereum").unwrap();
            assert_eq!(
                trace,
                AccountTrace::Remote(vec![
                    TruncatedChainId::from_str("bitcoin").unwrap(),
                    TruncatedChainId::from_str("ethereum").unwrap()
                ])
            );
        }

        #[test]
        fn remote_multi_multi_works() {
            let trace = AccountTrace::from_str("bitcoin>ethereum>cosmos").unwrap();
            assert_eq!(
                trace,
                AccountTrace::Remote(vec![
                    TruncatedChainId::from_str("bitcoin").unwrap(),
                    TruncatedChainId::from_str("ethereum").unwrap(),
                    TruncatedChainId::from_str("cosmos").unwrap(),
                ])
            );
        }

        // now test failures
        #[test]
        fn local_empty_fails() {
            AccountTrace::from_str("").unwrap_err();
        }

        #[test]
        fn local_too_short_fails() {
            AccountTrace::from_str("a").unwrap_err();
        }

        #[test]
        fn local_too_long_fails() {
            AccountTrace::from_str(&"a".repeat(MAX_CHAIN_NAME_LENGTH + 1)).unwrap_err();
        }

        #[test]
        fn local_uppercase_fails() {
            AccountTrace::from_str("AAAAA").unwrap_err();
        }

        #[test]
        fn local_non_alphanumeric_fails() {
            AccountTrace::from_str("a!aoeuoau").unwrap_err();
        }
    }

    mod key {
        use super::*;

        fn mock_key() -> AccountTrace {
            AccountTrace::Remote(vec![TruncatedChainId::from_str("bitcoin").unwrap()])
        }

        #[test]
        fn storage_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<&AccountTrace, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key, &42069).unwrap();

            assert_eq!(map.load(deps.as_ref().storage, &key).unwrap(), 42069);

            let items = map
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 1);
            assert_eq!(items[0], (key, 42069));
        }

        #[test]
        fn composite_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<(&AccountTrace, Addr), u64> = Map::new("map");

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("larry")),
                &42069,
            )
            .unwrap();

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("jake")),
                &69420,
            )
            .unwrap();

            let items = map
                .prefix(&key)
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(items[0], (Addr::unchecked("jake"), 69420));
            assert_eq!(items[1], (Addr::unchecked("larry"), 42069));
        }
    }
}
