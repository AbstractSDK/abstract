use std::{fmt::Display, str::FromStr};

use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::constants::ATTRIBUTE_DELIMITER;

/// Key to get the Address of a contract
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
// Need hash for ans scraper
#[cfg_attr(not(target_arch = "wasm32"), derive(Hash))]
pub struct UncheckedContractEntry {
    pub protocol: String,
    pub contract: String,
}

impl UncheckedContractEntry {
    pub fn new<T: ToString, R: ToString>(protocol: T, contract: R) -> Self {
        Self {
            protocol: protocol.to_string(),
            contract: contract.to_string(),
        }
    }
    pub fn check(self) -> ContractEntry {
        ContractEntry {
            contract: self.contract.to_ascii_lowercase(),
            protocol: self.protocol.to_ascii_lowercase(),
        }
    }
}

impl From<ContractEntry> for UncheckedContractEntry {
    fn from(contract_entry: ContractEntry) -> Self {
        Self {
            protocol: contract_entry.protocol,
            contract: contract_entry.contract,
        }
    }
}

impl TryFrom<&str> for UncheckedContractEntry {
    type Error = StdError;
    /// Try from a string slice like "protocol:contract_name"
    fn try_from(entry: &str) -> Result<Self, Self::Error> {
        let Some((protocol, contract_name)) = entry.split_once(ATTRIBUTE_DELIMITER) else {
            return Err(StdError::generic_err(
                "contract entry should be formatted as \"protocol:contract_name\".",
            ));
        };
        Ok(Self::new(protocol, contract_name))
    }
}

/// Key to get the Address of a contract
/// Use [`UncheckedContractEntry`] to construct this type.  
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
pub struct ContractEntry {
    pub protocol: String,
    pub contract: String,
}

impl FromStr for ContractEntry {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UncheckedContractEntry::try_from(s).map(Into::into)
    }
}

impl From<UncheckedContractEntry> for ContractEntry {
    fn from(entry: UncheckedContractEntry) -> Self {
        entry.check()
    }
}

impl Display for ContractEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{ATTRIBUTE_DELIMITER}{}", self.protocol, self.contract)
    }
}

impl PrimaryKey<'_> for &ContractEntry {
    type Prefix = String;

    type SubPrefix = ();

    type Suffix = String;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.protocol.key();
        keys.extend(self.contract.key());
        keys
    }
}

impl Prefixer<'_> for &ContractEntry {
    fn prefix(&self) -> Vec<Key> {
        let mut res = self.protocol.prefix();
        res.extend(self.contract.prefix());
        res
    }
}

impl KeyDeserialize for &ContractEntry {
    type Output = ContractEntry;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(ContractEntry {
            protocol: String::from_vec(tu)?,
            contract: String::from_vec(u)?,
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

        fn mock_key() -> ContractEntry {
            ContractEntry {
                protocol: "abstract".to_string(),
                contract: "rocket-ship".to_string(),
            }
        }

        fn mock_keys() -> (ContractEntry, ContractEntry, ContractEntry) {
            (
                ContractEntry {
                    protocol: "abstract".to_string(),
                    contract: "sailing-ship".to_string(),
                },
                ContractEntry {
                    protocol: "abstract".to_string(),
                    contract: "rocket-ship".to_string(),
                },
                ContractEntry {
                    protocol: "shitcoin".to_string(),
                    contract: "pump'n dump".to_string(),
                },
            )
        }

        #[coverage_helper::test]
        fn storage_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<&ContractEntry, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key, &42069).unwrap();

            assert_eq!(map.load(deps.as_ref().storage, &key).unwrap(), 42069);

            let items = map
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 1);
            assert_eq!(items[0], (key, 42069));
        }

        #[coverage_helper::test]
        fn composite_key_works() {
            let mut deps = mock_dependencies();
            let key = mock_key();
            let map: Map<(&ContractEntry, Addr), u64> = Map::new("map");

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

        #[coverage_helper::test]
        fn partial_key_works() {
            let mut deps = mock_dependencies();
            let (key1, key2, key3) = mock_keys();
            let map: Map<&ContractEntry, u64> = Map::new("map");

            map.save(deps.as_mut().storage, &key1, &42069).unwrap();

            map.save(deps.as_mut().storage, &key2, &69420).unwrap();

            map.save(deps.as_mut().storage, &key3, &999).unwrap();

            let items = map
                .prefix("abstract".to_string())
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .map(|item| item.unwrap())
                .collect::<Vec<_>>();

            assert_eq!(items.len(), 2);
            assert_eq!(items[0], ("rocket-ship".to_string(), 69420));
            assert_eq!(items[1], ("sailing-ship".to_string(), 42069));
        }

        #[coverage_helper::test]
        fn test_contract_entry_from_str() {
            let contract_entry_str = "abstract:rocket-ship";
            let contract_entry = ContractEntry::from_str(contract_entry_str).unwrap();

            assert_eq!(contract_entry.protocol, "abstract");
            assert_eq!(contract_entry.contract, "rocket-ship");

            let contract_entry_str = "foo:>420/,:z/69";
            let contract_entry = ContractEntry::from_str(contract_entry_str).unwrap();

            assert_eq!(contract_entry.protocol, "foo");
            assert_eq!(contract_entry.contract, ">420/,:z/69");

            // Wrong formatting
            let contract_entry_str = "shitcoin/,>rocket-ship";
            let err = ContractEntry::from_str(contract_entry_str).unwrap_err();

            assert_eq!(
                err,
                StdError::generic_err(
                    "contract entry should be formatted as \"protocol:contract_name\".",
                )
            );
        }

        #[coverage_helper::test]
        fn test_contract_entry_to_string() {
            let contract_entry_str = "abstract:app";
            let contract_entry = ContractEntry::from_str(contract_entry_str).unwrap();

            assert_eq!(contract_entry.to_string(), contract_entry_str);
        }
    }
}
