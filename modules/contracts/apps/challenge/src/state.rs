use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_std::Addr;
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
pub struct ChallengeEntryUpdate {
    pub name: Option<String>,
    pub collateral: Option<Penalty>,
    pub description: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub enum UpdateFriendsOpKind {
    Add,
    Remove,
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
    pub address: Addr,
    pub name: String,
}

#[cosmwasm_schema::cw_serde]
pub struct Vote {
    pub voter: String,
    pub vote: Option<bool>,
}

#[cosmwasm_schema::cw_serde]
pub struct CheckIn {
    pub last_checked_in: Option<u64>,
    pub next_check_in_by: u64, //block number
    pub metadata: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const ADMIN: Item<Addr> = Item::new("admin");
pub const CHALLENGE_LIST: Map<u64, ChallengeEntry> = Map::new("challenge_list");
pub const CHALLENGE_FRIENDS: Map<u64, Vec<Friend>> = Map::new("challenge_friends");
pub const VOTES: Map<u64, Vec<Vote>> = Map::new("votes");
// use a snapshot map?
pub const DAILY_CHECK_INS: Map<u64, CheckIn> = Map::new("daily_checkins");
