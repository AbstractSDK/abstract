use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_std::{Decimal, Uint128};
use croncat_app::croncat_integration_utils::CronCatInterval;
use cw_storage_plus::{Item, Map};

use crate::msg::Frequency;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
    pub forfeit_amount: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntry {
    pub name: String,
    pub source_asset: OfferAsset,
    pub frequency: Frequency,
}

#[cosmwasm_schema::cw_serde]
pub struct Friend {
    pub address: String,
    pub name: String,
}

#[cosmwasm_schema::cw_serde]
pub struct Vote {
    pub voter: String,
    pub vote: bool,
    pub challenge_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const CHALLENGE_LIST: Map<String, ChallengeEntry> = Map::new("challenge_list");
pub const CHALLENGE_FRIENDS: Map<(String, u64), Friend> = Map::new("challenge_friends");
pub const VOTES: Map<u64, Vec<Vote>> = Map::new("votes");
pub const DAILY_CHECKINS: Map<String, CronCatInterval> = Map::new("daily_checkins");
