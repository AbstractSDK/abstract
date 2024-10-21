use std::fmt::Display;

use cosmwasm_std::StdResult;
use cw_storage_plus::{KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    constants::{ASSET_DELIMITER, TYPE_DELIMITER},
    objects::AssetEntry,
};

type DexName = String;

/// The key for an asset pairing
/// Consists of the two assets and the dex name
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
pub struct DexAssetPairing<Asset = AssetEntry>((Asset, Asset, DexName));

impl<Asset> DexAssetPairing<Asset> {
    pub fn new(asset_x: Asset, asset_y: Asset, dex_name: &str) -> Self {
        Self((asset_x, asset_y, str::to_ascii_lowercase(dex_name)))
    }

    pub fn asset_x(&self) -> &Asset {
        &self.0 .0
    }

    pub fn asset_y(&self) -> &Asset {
        &self.0 .1
    }

    pub fn dex(&self) -> &str {
        &self.0 .2
    }
}

impl Display for DexAssetPairing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{TYPE_DELIMITER}{}{ASSET_DELIMITER}{}",
            self.dex(),
            self.asset_x(),
            self.asset_y()
        )
    }
}

impl<'a> PrimaryKey<'a> for &DexAssetPairing {
    type Prefix = (&'a AssetEntry, &'a AssetEntry);
    type SubPrefix = &'a AssetEntry;
    type Suffix = DexName;
    type SuperSuffix = (&'a AssetEntry, DexName);

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        <(AssetEntry, AssetEntry, DexName)>::key(&self.0)
    }
}

impl<'a> Prefixer<'a> for &DexAssetPairing {
    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
        <(AssetEntry, AssetEntry, DexName)>::prefix(&self.0)
    }
}

impl KeyDeserialize for &DexAssetPairing {
    type Output = DexAssetPairing;
    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(DexAssetPairing(
            <(&AssetEntry, &AssetEntry, &DexName)>::from_vec(value)?,
        ))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use cosmwasm_std::{testing::mock_dependencies, Addr, Order};
    use cw_storage_plus::Map;

    use super::*;
    use crate::objects::{AnsEntryConvertor, LpToken, PoolReference, UniquePoolId};

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

    #[coverage_helper::test]
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

    #[coverage_helper::test]
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

    #[coverage_helper::test]
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

    #[coverage_helper::test]
    fn try_from_lp_token() {
        let lp = LpToken::new("junoswap", vec!["juno".to_string(), "osmo".to_string()]);

        let key = AnsEntryConvertor::new(lp).dex_asset_pairing().unwrap();

        assert_eq!(
            key,
            DexAssetPairing::new("juno".into(), "osmo".into(), "junoswap")
        );
    }

    #[coverage_helper::test]
    fn display() {
        let key = DexAssetPairing::new("juno".into(), "osmo".into(), "junoswap");
        assert_eq!(key.to_string(), "junoswap/juno,osmo");
    }
}
