use abstract_dex_adapter::msg::OfferAsset;
use cw_storage_plus::{Item, Map};

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntry {
    pub name: String,
    pub collateral: Penalty,
    pub description: String,
}

#[cosmwasm_schema::cw_serde]
pub enum Penalty {
    FixedAmount {
        asset: OfferAsset,
    },
    Daily {
        asset: OfferAsset,
        split_between_friends: bool,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct Friend {
    pub address: String,
    pub name: String,
}

#[cosmwasm_schema::cw_serde]
pub struct Vote {
    pub voter: String,
    pub vote: Option<bool>,
    pub challenge_id: String,
}

#[cosmwasm_schema::cw_serde]
pub struct CheckIn {
    pub last_checked_in: Option<String>,
    pub next_check_in_by: u64, //block number
    pub metadata: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const CHALLENGE_LIST: Map<String, ChallengeEntry> = Map::new("challenge_list");
pub const CHALLENGE_FRIENDS: Map<String, Vec<Friend>> = Map::new("challenge_friends");
pub const VOTES: Map<String, Vec<Vote>> = Map::new("votes");
// use a snapshot map?
pub const DAILY_CHECK_INS: Map<String, CheckIn> = Map::new("daily_checkins");
