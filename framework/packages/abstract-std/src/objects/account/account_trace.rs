use std::fmt::Display;

use super::account_id::deser::split_first_key;
use cosmwasm_std::{ensure, Env, StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::{constants::CHAIN_DELIMITER, objects::TruncatedChainId, AbstractError};

pub const MAX_TRACE_LENGTH: u16 = 6;
pub(crate) const LOCAL: &str = "local";

/// The identifier of chain that triggered the account creation
#[cosmwasm_schema::cw_serde]
pub enum AccountTrace {
    Local,
    // path of the chains that triggered the account creation
    Remote(Vec<TruncatedChainId>),
}

pub const ACCOUNT_TRACE_KEY_PLACEHOLDER: &[u8] = &[];

impl KeyDeserialize for &AccountTrace {
    type Output = AccountTrace;
    const KEY_ELEMS: u16 = AccountTrace::KEY_ELEMS;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        let mut trace = vec![];
        // We parse the whole data for the MAX_TRACE_LENGTH keys
        let mut value = value.as_ref();
        for i in 0..MAX_TRACE_LENGTH - 1 {
            let (current_chain, remainder) = split_first_key(1, value)?;
            value = remainder;
            if current_chain == ACCOUNT_TRACE_KEY_PLACEHOLDER {
                continue;
            }
            let chain = String::from_utf8(current_chain)?;
            if i == 0 && chain == "local" {
                return Ok(AccountTrace::Local);
            }
            trace.push(TruncatedChainId::from_string(chain).unwrap())
        }

        Ok(AccountTrace::Remote(trace))
    }
}

impl KeyDeserialize for AccountTrace {
    type Output = AccountTrace;
    const KEY_ELEMS: u16 = MAX_TRACE_LENGTH;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        <&AccountTrace>::from_vec(value)
    }
}

impl<'a> PrimaryKey<'a> for AccountTrace {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut serialization_result = match self {
            AccountTrace::Local => LOCAL.key(),
            AccountTrace::Remote(chain_name) => chain_name
                .iter()
                .flat_map(|c| c.str_ref().key())
                .collect::<Vec<Key>>(),
        };
        for _ in serialization_result.len()..(MAX_TRACE_LENGTH as usize) {
            serialization_result.extend(ACCOUNT_TRACE_KEY_PLACEHOLDER.key());
        }
        serialization_result
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
                    chain_trace.len() <= MAX_TRACE_LENGTH as usize,
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
        account_trace_from_str(&trace)
    }

    pub(crate) fn from_str(trace: &str) -> Result<Self, AbstractError> {
        let acc = account_trace_from_str(trace);
        acc.verify()?;
        Ok(acc)
    }
}

impl TryFrom<&str> for AccountTrace {
    type Error = AbstractError;

    fn try_from(trace: &str) -> Result<Self, Self::Error> {
        AccountTrace::from_str(trace)
    }
}

fn account_trace_from_str(trace: &str) -> AccountTrace {
    if trace == LOCAL {
        AccountTrace::Local
    } else {
        let rev_trace: Vec<_> = trace
            // DoubleEndedSearcher implemented for char, but not for "str"
            .split(CHAIN_DELIMITER.chars().next().unwrap())
            .map(TruncatedChainId::_from_str)
            .rev()
            .collect();
        AccountTrace::Remote(rev_trace)
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
                    .rev()
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

        #[coverage_helper::test]
        fn local_works() {
            let trace = AccountTrace::from_str(LOCAL).unwrap();
            assert_eq!(trace, AccountTrace::Local);
        }

        #[coverage_helper::test]
        fn remote_works() {
            let trace = AccountTrace::from_str("bitcoin").unwrap();
            assert_eq!(
                trace,
                AccountTrace::Remote(vec![TruncatedChainId::from_str("bitcoin").unwrap()])
            );
        }

        #[coverage_helper::test]
        fn remote_multi_works() {
            // Here the account originates from ethereum and was then bridged to bitcoin
            let trace = AccountTrace::from_str("bitcoin>ethereum").unwrap();
            assert_eq!(
                trace,
                // The trace vector pushes the last chains last
                AccountTrace::Remote(vec![
                    TruncatedChainId::from_str("ethereum").unwrap(),
                    TruncatedChainId::from_str("bitcoin").unwrap(),
                ])
            );
        }

        #[coverage_helper::test]
        fn remote_multi_multi_works() {
            // Here the account originates from cosmos, and was then bridged to ethereum and was then bridged to bitcoin
            let trace = AccountTrace::from_str("bitcoin>ethereum>cosmos").unwrap();
            assert_eq!(
                trace,
                // The trace vector pushes the last chains last
                AccountTrace::Remote(vec![
                    TruncatedChainId::from_str("cosmos").unwrap(),
                    TruncatedChainId::from_str("ethereum").unwrap(),
                    TruncatedChainId::from_str("bitcoin").unwrap(),
                ])
            );
        }

        // now test failures
        #[coverage_helper::test]
        fn local_empty_fails() {
            AccountTrace::from_str("").unwrap_err();
        }

        #[coverage_helper::test]
        fn local_too_short_fails() {
            AccountTrace::from_str("a").unwrap_err();
        }

        #[coverage_helper::test]
        fn local_too_long_fails() {
            AccountTrace::from_str(&"a".repeat(MAX_CHAIN_NAME_LENGTH + 1)).unwrap_err();
        }

        #[coverage_helper::test]
        fn local_uppercase_fails() {
            AccountTrace::from_str("AAAAA").unwrap_err();
        }

        #[coverage_helper::test]
        fn local_non_alphanumeric_fails() {
            AccountTrace::from_str("a!aoeuoau").unwrap_err();
        }
    }

    mod key {
        use super::*;

        fn mock_key() -> AccountTrace {
            AccountTrace::Remote(vec![TruncatedChainId::from_str("bitcoin").unwrap()])
        }

        fn mock_local_key() -> AccountTrace {
            AccountTrace::Remote(vec![])
        }

        fn mock_multi_hop_key() -> AccountTrace {
            AccountTrace::Remote(vec![
                TruncatedChainId::from_str("bitcoin").unwrap(),
                TruncatedChainId::from_str("atom").unwrap(),
                TruncatedChainId::from_str("foo").unwrap(),
            ])
        }

        #[coverage_helper::test]
        fn storage_key_works() {
            let mut deps = mock_dependencies();
            let local_key = mock_local_key();
            let key = mock_key();
            let multihop_key = mock_multi_hop_key();
            let map: Map<&AccountTrace, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &local_key, &159784)
                .unwrap();
            map.save(deps.as_mut().storage, &key, &42069).unwrap();
            map.save(deps.as_mut().storage, &multihop_key, &69420)
                .unwrap();

            assert_eq!(map.load(deps.as_ref().storage, &local_key).unwrap(), 159784);
            assert_eq!(map.load(deps.as_ref().storage, &key).unwrap(), 42069);
            assert_eq!(
                map.load(deps.as_ref().storage, &multihop_key).unwrap(),
                69420
            );

            let items = map
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 3);
            assert_eq!(items[0], (local_key, 159784));
            assert_eq!(items[1], (key, 42069));
            assert_eq!(items[2], (multihop_key, 69420));
        }

        #[coverage_helper::test]
        fn composite_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let multihop_key = mock_multi_hop_key();
            let map: Map<(&AccountTrace, Addr), u64> = Map::new("map");

            map.save(
                deps.as_mut().storage,
                (&key, Addr::unchecked("larry")),
                &42069,
            )
            .unwrap();
            map.save(
                deps.as_mut().storage,
                (&multihop_key, Addr::unchecked("larry")),
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
