use std::array::TryFromSliceError;

use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::{IntKey, Item, Key, KeyDeserialize, Map, PrimaryKey};

use crate::msg::Frequency;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
    pub dca_creation_amount: Uint128,
    pub refill_threshold: Uint128,
    pub max_spread: Decimal,
}

#[cosmwasm_schema::cw_serde]
pub struct DCAEntry {
    pub source_asset: OfferAsset,
    pub target_asset: AssetEntry,
    pub frequency: Frequency,
    pub dex: DexName,
}

#[cosmwasm_schema::cw_serde]
#[derive(Copy, Default)]
pub struct DCAId(pub u64);

impl DCAId {
    pub fn next_id(self) -> Self {
        // You won't overflow it accidentally
        Self(self.0 + 1)
    }
}

impl<'a> PrimaryKey<'a> for DCAId {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        vec![Key::Val64(self.0.to_cw_bytes())]
    }
}

impl KeyDeserialize for DCAId {
    type Output = u64;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        Ok(u64::from_cw_bytes(value.as_slice().try_into().map_err(
            |err: TryFromSliceError| cosmwasm_std::StdError::generic_err(err.to_string()),
        )?))
    }
}

// Convert it to croncat tag
impl From<DCAId> for String {
    fn from(DCAId(id): DCAId) -> Self {
        format!("dca_{id}")
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<DCAId> = Item::new("next_id");
pub const DCA_LIST: Map<DCAId, DCAEntry> = Map::new("dca_list");
