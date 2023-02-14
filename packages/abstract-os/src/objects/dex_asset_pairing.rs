use crate::objects::lp_token::LpToken;
use crate::objects::AssetEntry;
use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt::Display};

type DexName = String;

/// The key for an asset pairing
/// Consists of the two assets and the dex name
/// TODO: what if we made keys equal based on the two assets either way?
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
pub struct DexAssetPairing((AssetEntry, AssetEntry, DexName));

impl DexAssetPairing {
    pub fn new(asset_x: AssetEntry, asset_y: AssetEntry, dex_name: &str) -> Self {
        Self((asset_x, asset_y, str::to_ascii_lowercase(dex_name)))
    }

    pub fn asset_x(&self) -> &AssetEntry {
        &self.0 .0
    }

    pub fn asset_y(&self) -> &AssetEntry {
        &self.0 .1
    }

    pub fn dex(&self) -> &str {
        &self.0 .2
    }
}

impl TryFrom<AssetEntry> for DexAssetPairing {
    type Error = StdError;

    fn try_from(asset_entry: AssetEntry) -> Result<Self, Self::Error> {
        let lp_token: LpToken = asset_entry.try_into()?;
        let mut assets = lp_token.assets;
        // assets should already be sorted, but just in case
        assets.sort();
        assets.reverse();

        Ok(Self::new(
            assets.pop().unwrap(),
            assets.pop().unwrap(),
            lp_token.dex.as_str(),
        ))
    }
}

impl Display for DexAssetPairing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}-{}", self.dex(), self.asset_x(), self.asset_y())
    }
}

impl<'a> PrimaryKey<'a> for &DexAssetPairing {
    type Prefix = (&'a AssetEntry, &'a AssetEntry);
    type SubPrefix = &'a AssetEntry;
    type Suffix = DexName;
    type SuperSuffix = (&'a AssetEntry, DexName);

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut key = self.0 .0 .0.key();
        key.extend(self.0 .1 .0.key());
        key.extend(self.0 .2.key());
        key
    }
}

impl<'a> Prefixer<'a> for &DexAssetPairing {
    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
        let mut res = self.0 .0 .0.prefix();
        res.extend(self.0 .1 .0.prefix().into_iter());
        res.extend(self.0 .2.prefix().into_iter());
        res
    }
}

fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}

/// @todo: use existing method for triple tuple
impl KeyDeserialize for &DexAssetPairing {
    type Output = DexAssetPairing;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tuv = value.split_off(2);
        let t_len = parse_length(&value)?;
        let mut len_uv = tuv.split_off(t_len);

        let mut uv = len_uv.split_off(2);
        let u_len = parse_length(&len_uv)?;
        let v = uv.split_off(u_len);

        Ok(DexAssetPairing::new(
            String::from_vec(tuv)?.into(),
            String::from_vec(uv)?.into(),
            &String::from_vec(v)?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::objects::{PoolReference, UniquePoolId};
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    fn mock_key() -> DexAssetPairing {
        DexAssetPairing::new("juno".into(), "osmo".into(), "junoswap")
    }

    fn mock_keys() -> (DexAssetPairing, DexAssetPairing, DexAssetPairing) {
        (
            DexAssetPairing::new("juno".into(), "osmo".into(), "junoswap"),
            DexAssetPairing::new("juno".into(), "osmo".into(), "osmosis"),
            DexAssetPairing::new("osmo".into(), "usdt".into(), "osmosis"),
        )
    }

    fn mock_pool_ref(id: u64, name: &str) -> PoolReference {
        PoolReference {
            unique_id: UniquePoolId::new(id),
            pool_address: Addr::unchecked(name).into(),
        }
    }

    #[test]
    fn storage_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<&DexAssetPairing, u64> = Map::new("map");

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
        let map: Map<(&DexAssetPairing, Addr), Vec<PoolReference>> = Map::new("map");

        let ref_1 = mock_pool_ref(1, "larry0x");
        let ref_2 = mock_pool_ref(2, "stablechen");

        map.save(
            deps.as_mut().storage,
            (&key, Addr::unchecked("astroport")),
            &vec![ref_1.clone()],
        )
        .unwrap();

        map.save(
            deps.as_mut().storage,
            (&key, Addr::unchecked("terraswap")),
            &vec![ref_2.clone()],
        )
        .unwrap();

        let items = map
            .prefix(&key)
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], (Addr::unchecked("astroport"), vec![ref_1]));
        assert_eq!(items[1], (Addr::unchecked("terraswap"), vec![ref_2]));
    }

    #[test]
    fn partial_key_works() {
        let mut deps = mock_dependencies();
        let (key1, key2, key3) = mock_keys();
        let map: Map<&DexAssetPairing, u64> = Map::new("map");

        map.save(deps.as_mut().storage, &key1, &42069).unwrap();

        map.save(deps.as_mut().storage, &key2, &69420).unwrap();

        map.save(deps.as_mut().storage, &key3, &999).unwrap();

        let items = map
            .prefix((&"juno".into(), &"osmo".into()))
            .range(deps.as_ref().storage, None, None, Order::Ascending)
            .map(|item| item.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("junoswap".to_string(), 42069));
        assert_eq!(items[1], ("osmosis".to_string(), 69420));
    }

    #[test]
    fn try_from_lp_token() {
        let lp_token = AssetEntry::new("junoswap/juno,osmo");

        let key = DexAssetPairing::try_from(lp_token).unwrap();

        assert_eq!(
            key,
            DexAssetPairing::new("juno".into(), "osmo".into(), "junoswap")
        );
    }
}
