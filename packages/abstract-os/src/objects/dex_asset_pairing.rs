use std::{convert::TryInto, fmt::Display};

use cosmwasm_std::{StdError, StdResult};

use crate::objects::lp_token::LpToken;
use crate::objects::AssetEntry;
use cw_storage_plus::{KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

type DexName = String;

/// The key for an asset pairing
/// Consists of the two assets and the dex name
/// TODO: what if we made keys equal based on the two assets either way?
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
pub struct DexAssetPairing((String, String, DexName));

impl DexAssetPairing {
    pub fn new(asset_x: &str, asset_y: &str, dex_name: &str) -> Self {
        Self((
            str::to_ascii_lowercase(asset_x),
            str::to_ascii_lowercase(asset_y),
            str::to_ascii_lowercase(dex_name),
        ))
    }

    pub fn asset_x(&self) -> &str {
        &self.0 .0
    }

    pub fn asset_y(&self) -> &str {
        &self.0 .1
    }

    pub fn dex(&self) -> &str {
        &self.0 .2
    }

    pub fn from_assets(dex_name: &str, assets: &mut [&AssetEntry; 2]) -> Self {
        assets.sort();
        Self::new(assets[0].as_str(), assets[1].as_str(), dex_name)
    }
}

impl TryFrom<AssetEntry> for DexAssetPairing {
    type Error = StdError;

    fn try_from(asset_entry: AssetEntry) -> Result<Self, Self::Error> {
        let lp_token: LpToken = asset_entry.try_into()?;
        let mut assets = lp_token.assets;
        // assets should already be sorted, but just in case
        assets.sort();

        Ok(Self::new(
            assets[0].as_str(),
            assets[1].as_str(),
            lp_token.dex.as_str(),
        ))
    }
}

impl From<(String, String, String)> for DexAssetPairing {
    fn from((asset_x, asset_y, dex_name): (String, String, String)) -> Self {
        Self::new(&asset_x, &asset_y, &dex_name)
    }
}

impl Display for DexAssetPairing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}-{}", self.dex(), self.asset_x(), self.asset_y())
    }
}

impl<'a> PrimaryKey<'a> for DexAssetPairing {
    type Prefix = (String, String);
    type SubPrefix = String;
    type Suffix = DexName;
    type SuperSuffix = (String, DexName);

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.key()
    }
}

impl<'a> Prefixer<'a> for DexAssetPairing {
    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
        self.0.prefix()
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
impl KeyDeserialize for DexAssetPairing {
    type Output = DexAssetPairing;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tuv = value.split_off(2);
        let t_len = parse_length(&value)?;
        let mut len_uv = tuv.split_off(t_len);

        let mut uv = len_uv.split_off(2);
        let u_len = parse_length(&len_uv)?;
        let v = uv.split_off(u_len);

        Ok((
            String::from_vec(tuv)?,
            String::from_vec(uv)?,
            String::from_vec(v)?,
        )
            .into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::objects::{PoolReference, UniquePoolId};
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    fn mock_key() -> DexAssetPairing {
        DexAssetPairing::new("juno", "osmo", "junoswap")
    }

    fn mock_keys() -> (DexAssetPairing, DexAssetPairing, DexAssetPairing) {
        (
            DexAssetPairing::new("juno", "osmo", "junoswap"),
            DexAssetPairing::new("juno", "osmo", "osmosis"),
            DexAssetPairing::new("osmo", "usdt", "osmosis"),
        )
    }

    fn mock_pool_ref(id: u64, name: &str) -> PoolReference {
        PoolReference {
            id: UniquePoolId::new(id),
            pool_id: Addr::unchecked(name).into(),
        }
    }

    #[test]
    fn storage_key_works() {
        let mut deps = mock_dependencies();
        let key = mock_key();
        let map: Map<DexAssetPairing, u64> = Map::new("map");

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
        let map: Map<(DexAssetPairing, Addr), Vec<PoolReference>> = Map::new("map");

        let ref_1 = mock_pool_ref(1, "larry0x");
        let ref_2 = mock_pool_ref(2, "stablechen");

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("astroport")),
            &vec![ref_1.clone()],
        )
        .unwrap();

        map.save(
            deps.as_mut().storage,
            (key.clone(), Addr::unchecked("terraswap")),
            &vec![ref_2.clone()],
        )
        .unwrap();

        let items = map
            .prefix(key)
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
        let map: Map<DexAssetPairing, u64> = Map::new("map");

        map.save(deps.as_mut().storage, key1, &42069).unwrap();

        map.save(deps.as_mut().storage, key2, &69420).unwrap();

        map.save(deps.as_mut().storage, key3, &999).unwrap();

        let items = map
            .prefix(("juno".to_string(), "osmo".to_string()))
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

        assert_eq!(key, DexAssetPairing::new("juno", "osmo", "junoswap"));
    }
}
