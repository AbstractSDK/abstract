use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt::Display};

/// Key to get the Address of a connected_chain
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
pub struct UncheckedChannelEntry {
    pub connected_chain: String,
    pub protocol: String,
}

impl UncheckedChannelEntry {
    pub fn new<T: ToString>(connected_chain: T, protocol: T) -> Self {
        Self {
            protocol: protocol.to_string(),
            connected_chain: connected_chain.to_string(),
        }
    }
    pub fn check(self) -> ChannelEntry {
        ChannelEntry {
            connected_chain: self.connected_chain.to_ascii_lowercase(),
            protocol: self.protocol.to_ascii_lowercase(),
        }
    }
}

impl TryFrom<String> for UncheckedChannelEntry {
    type Error = StdError;
    fn try_from(entry: String) -> Result<Self, Self::Error> {
        let composite: Vec<&str> = entry.split('/').collect();
        if composite.len() != 2 {
            return Err(StdError::generic_err(
                "connected_chain entry should be formatted as \"connected_chain_name/protocol\".",
            ));
        }
        Ok(Self::new(composite[0], composite[1]))
    }
}

/// Key to get the Address of a connected_chain
/// Use [`UncheckedChannelEntry`] to construct this type.  
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
pub struct ChannelEntry {
    pub connected_chain: String,
    pub protocol: String,
}

impl Display for ChannelEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.connected_chain, self.protocol)
    }
}

impl<'a> PrimaryKey<'a> for ChannelEntry {
    type Prefix = String;

    type SubPrefix = ();

    type Suffix = String;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.connected_chain.key();
        keys.extend(self.protocol.key());
        keys
    }
}

impl<'a> Prefixer<'a> for ChannelEntry {
    fn prefix(&self) -> Vec<Key> {
        let mut res = self.connected_chain.prefix();
        res.extend(self.protocol.prefix().into_iter());
        res
    }
}

impl KeyDeserialize for ChannelEntry {
    type Output = Self;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(Self {
            connected_chain: String::from_vec(tu)?,
            protocol: String::from_vec(u)?,
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
    use super::*;
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    fn mock_key() -> ChannelEntry {
        ChannelEntry {
            connected_chain: "osmosis".to_string(),
            protocol: "ics20".to_string(),
        }
    }

    fn mock_keys() -> (ChannelEntry, ChannelEntry, ChannelEntry) {
        (
            ChannelEntry {
                connected_chain: "osmosis".to_string(),
                protocol: "ics20".to_string(),
            },
            ChannelEntry {
                connected_chain: "osmosis".to_string(),
                protocol: "ics".to_string(),
            },
            ChannelEntry {
                connected_chain: "cosmos".to_string(),
                protocol: "abstract".to_string(),
            },
        )
    }

    #[test]
    fn storage_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<ChannelEntry, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key.clone(), &42069)
            .unwrap();

        assert_eq!(map.load(deps.as_ref().storage, key.clone()).unwrap(), 42069);

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
        let map: Map<(ChannelEntry, Addr), u64> = Map::new("map");

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("larry")),
            &42069,
        )
        .unwrap();

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("jake")),
            &69420,
        )
        .unwrap();

        let items = map
            .prefix(key)
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
        let map: Map<ChannelEntry, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key1, &42069).unwrap();

        map.save(deps.as_mut().storage, key2, &69420).unwrap();

        map.save(deps.as_mut().storage, key3, &999).unwrap();

        let items = map
            .prefix("osmosis".to_string())
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("ics".to_string(), 69420));
        assert_eq!(items[1], ("ics20".to_string(), 42069));
    }
}
