use std::fmt::Display;

use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use super::{account_trace::AccountTrace, AccountSequence};
use crate::{objects::TruncatedChainId, AbstractError};

/// Unique identifier for an account.
/// On each chain this is unique.
#[cosmwasm_schema::cw_serde]
pub struct AccountId {
    /// Sequence of the chain that triggered the IBC account creation
    /// `AccountTrace::Local` if the account was created locally
    /// Example: Account created on Juno which has an abstract interchain account on Osmosis,
    /// which in turn creates an interchain account on Terra -> `AccountTrace::Remote(vec!["juno", "osmosis"])`
    trace: AccountTrace,
    /// Unique identifier for the accounts create on a local chain.
    /// Is reused when creating an interchain account.
    seq: AccountSequence,
}

impl Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.trace, self.seq)
    }
}

impl AccountId {
    pub fn new(seq: AccountSequence, trace: AccountTrace) -> Result<Self, AbstractError> {
        trace.verify()?;
        Ok(Self { seq, trace })
    }

    pub fn local(seq: AccountSequence) -> Self {
        Self {
            seq,
            trace: AccountTrace::Local,
        }
    }

    pub fn remote(
        seq: AccountSequence,
        trace: Vec<TruncatedChainId>,
    ) -> Result<Self, AbstractError> {
        let trace = AccountTrace::Remote(trace);
        trace.verify()?;
        Ok(Self { seq, trace })
    }

    /// Construct the `AccountId` for an account on a remote chain based on the current Account.
    /// Will pop the trace if the destination chain is the last chain in the trace.
    pub fn into_dest_account_id(
        mut self,
        src_chain: TruncatedChainId,
        dest_chain: TruncatedChainId,
    ) -> Self {
        match &mut self.trace {
            AccountTrace::Remote(ref mut chains) => {
                // if last account chain is the destination chain, pop
                if chains.last() != Some(&dest_chain) {
                    chains.push(src_chain);
                } else {
                    chains.pop();
                    // if the pop made the AccountId empty then we're targeting a local account.
                    if chains.is_empty() {
                        self.trace = AccountTrace::Local;
                    }
                }
            }
            AccountTrace::Local => {
                self.trace = AccountTrace::Remote(vec![src_chain]);
            }
        }
        self
    }

    /// **Does not verify input**. Used internally for testing
    pub const fn const_new(seq: AccountSequence, trace: AccountTrace) -> Self {
        Self { seq, trace }
    }

    pub fn seq(&self) -> AccountSequence {
        self.seq
    }

    pub fn trace(&self) -> &AccountTrace {
        &self.trace
    }

    pub fn trace_mut(&mut self) -> &mut AccountTrace {
        &mut self.trace
    }

    pub fn is_local(&self) -> bool {
        matches!(self.trace, AccountTrace::Local)
    }

    pub fn is_remote(&self) -> bool {
        !self.is_local()
    }

    /// Push the chain to the account trace
    pub fn push_chain(&mut self, chain: TruncatedChainId) {
        self.trace_mut().push_chain(chain)
    }

    pub fn decompose(self) -> (AccountTrace, AccountSequence) {
        (self.trace, self.seq)
    }
}

impl TryFrom<&str> for AccountId {
    type Error = AbstractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (trace_str, seq_str) = value
            .split_once('-')
            .ok_or(AbstractError::FormattingError {
                object: "AccountId".into(),
                expected: "trace-999".into(),
                actual: value.into(),
            })?;
        let seq: u32 = seq_str.parse().unwrap();
        if value.starts_with(super::account_trace::LOCAL) {
            Ok(AccountId {
                trace: AccountTrace::Local,
                seq,
            })
        } else {
            Ok(AccountId {
                trace: AccountTrace::from_string(trace_str.into()),
                seq,
            })
        }
    }
}

impl<'a> PrimaryKey<'a> for AccountId {
    type Prefix = AccountTrace;

    type SubPrefix = ();

    type Suffix = AccountSequence;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.trace.key();
        keys.extend(self.seq.key());
        keys
    }
}

impl<'a> Prefixer<'a> for AccountId {
    fn prefix(&self) -> Vec<Key> {
        self.key()
    }
}

impl KeyDeserialize for &AccountId {
    type Output = AccountId;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(AccountId {
            seq: AccountSequence::from_vec(u)?,
            trace: AccountTrace::from_string(String::from_vec(tu)?),
        })
    }
}

impl KeyDeserialize for AccountId {
    type Output = AccountId;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(AccountId {
            seq: AccountSequence::from_vec(u)?,
            trace: AccountTrace::from_string(String::from_vec(tu)?),
        })
    }
}

#[inline(always)]
fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    use super::*;

    mod key {
        use super::*;

        use std::str::FromStr;

        fn mock_key() -> AccountId {
            AccountId {
                seq: 1,
                trace: AccountTrace::Remote(vec![TruncatedChainId::from_str("bitcoin").unwrap()]),
            }
        }

        fn mock_keys() -> (AccountId, AccountId, AccountId) {
            (
                AccountId {
                    seq: 1,
                    trace: AccountTrace::Local,
                },
                AccountId {
                    seq: 1,
                    trace: AccountTrace::Remote(vec![
                        TruncatedChainId::from_str("ethereum").unwrap(),
                        TruncatedChainId::from_str("bitcoin").unwrap(),
                    ]),
                },
                AccountId {
                    seq: 2,
                    trace: AccountTrace::Remote(vec![
                        TruncatedChainId::from_str("ethereum").unwrap(),
                        TruncatedChainId::from_str("bitcoin").unwrap(),
                    ]),
                },
            )
        }

        #[test]
        fn storage_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<&AccountId, u64> = Map::new("map");

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
            let map: Map<(&AccountId, Addr), u64> = Map::new("map");

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

        #[test]
        fn partial_key_works() {
            let mut deps = mock_dependencies();
            let (key1, key2, key3) = mock_keys();
            let map: Map<&AccountId, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key1, &42069).unwrap();

            map.save(deps.as_mut().storage, &key2, &69420).unwrap();

            map.save(deps.as_mut().storage, &key3, &999).unwrap();

            let items = map
                .prefix(AccountTrace::Remote(vec![
                    TruncatedChainId::from_str("ethereum").unwrap(),
                    TruncatedChainId::from_str("bitcoin").unwrap(),
                ]))
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(items[0], (1, 69420));
            assert_eq!(items[1], (2, 999));
        }

        #[test]
        fn works_as_storage_key_with_multiple_chains_in_trace() {
            let mut deps = mock_dependencies();
            let key = AccountId {
                seq: 1,
                trace: AccountTrace::Remote(vec![
                    TruncatedChainId::from_str("ethereum").unwrap(),
                    TruncatedChainId::from_str("bitcoin").unwrap(),
                ]),
            };
            let map: Map<&AccountId, u64> = Map::new("map");

            let value = 1;
            map.save(deps.as_mut().storage, &key, &value).unwrap();

            assert_eq!(value, map.load(deps.as_ref().storage, &key).unwrap());
        }
    }

    mod try_from {
        // test that the try_from implementation works
        use super::*;

        #[test]
        fn works_with_local() {
            let account_id = AccountId::try_from("local-1").unwrap();
            assert_eq!(account_id.seq, 1);
            assert_eq!(account_id.trace, AccountTrace::Local);
        }

        #[test]
        fn works_with_remote() {
            let account_id = AccountId::try_from("ethereum>bitcoin-1").unwrap();
            assert_eq!(account_id.seq, 1);
            assert_eq!(
                account_id.trace,
                AccountTrace::Remote(vec![
                    TruncatedChainId::_from_str("ethereum"),
                    TruncatedChainId::_from_str("bitcoin"),
                ])
            );
        }

        #[test]
        fn works_with_remote_with_multiple_chains() {
            let account_id = AccountId::try_from("ethereum>bitcoin>cosmos-1").unwrap();
            assert_eq!(account_id.seq, 1);
            assert_eq!(
                account_id.trace,
                AccountTrace::Remote(vec![
                    TruncatedChainId::_from_str("ethereum"),
                    TruncatedChainId::_from_str("bitcoin"),
                    TruncatedChainId::_from_str("cosmos"),
                ])
            );
        }
    }
}
