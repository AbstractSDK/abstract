use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{IntKey, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::array::TryFromSliceError;
use std::{convert::TryInto, fmt::Display};

#[derive(
    Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Copy,
)]
pub struct UniquePoolId(u64);

impl UniquePoolId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl From<u64> for UniquePoolId {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl Display for UniquePoolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> PrimaryKey<'a> for UniquePoolId {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl<'a> Prefixer<'a> for UniquePoolId {
    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
        self.0.prefix()
    }
}

impl KeyDeserialize for UniquePoolId {
    type Output = Self;
    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(Self::from_cw_bytes(value.as_slice().try_into().map_err(
            |err: TryFromSliceError| StdError::generic_err(err.to_string()),
        )?))
    }
}

impl IntKey for UniquePoolId {
    type Buf = [u8; std::mem::size_of::<u64>()];

    #[inline]
    fn to_cw_bytes(&self) -> Self::Buf {
        self.0.to_be_bytes()
    }

    #[inline]
    fn from_cw_bytes(bytes: Self::Buf) -> Self {
        Self(u64::from_be_bytes(bytes))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    fn mock_key() -> UniquePoolId {
        UniquePoolId::new(1)
    }

    fn _mock_keys() -> (UniquePoolId, UniquePoolId, UniquePoolId) {
        (
            UniquePoolId::new(1),
            UniquePoolId::new(2),
            UniquePoolId::new(3),
        )
    }

    #[test]
    fn storage_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<UniquePoolId, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key, &42069).unwrap();

        assert_eq!(map.load(deps.as_ref().storage, key).unwrap(), 42069);

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
        let map: Map<(UniquePoolId, Addr), u64> = Map::new("map");

        map.save(
            deps.as_mut().storage,
            (key, Addr::unchecked("larry")),
            &42069,
        )
        .unwrap();

        map.save(
            deps.as_mut().storage,
            (key, Addr::unchecked("jake")),
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
    fn naked_64key_works() {
        let k: UniquePoolId = 4242u64.into();
        let path = k.key();
        assert_eq!(1, path.len());
        assert_eq!(4242u64.to_cw_bytes(), path[0].as_ref());
    }
}
