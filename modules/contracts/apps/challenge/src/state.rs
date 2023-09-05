use abstract_dex_adapter::msg::OfferAsset;
use chrono::Duration;
use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Timestamp, Uint128};
use cw_address_like::AddressLike;
use cw_storage_plus::{Item, Map};

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntry {
    pub name: String,
    pub collateral: OfferAsset,
    pub description: String,
    pub end: EndType,
    pub total_check_ins: Option<u128>,
    pub status: ChallengeStatus,
    pub admin_strikes: [bool; 3],
}

#[cosmwasm_schema::cw_serde]
pub enum EndType {
    Duration(DurationChoice),
    ExactTime(Timestamp),
}

#[cosmwasm_schema::cw_serde]
pub enum DurationChoice {
    Week,
    Month,
    Quarter,
}

impl ChallengeEntry {
    /// Creates a new challenge entry with the default status of Uninitialized and no admin strikes.
    pub fn new(name: String, collateral: OfferAsset, description: String, end: EndType) -> Self {
        ChallengeEntry {
            name,
            collateral,
            description,
            end,
            total_check_ins: None,
            status: ChallengeStatus::default(),
            admin_strikes: [false; 3],
        }
    }
    /// Converts the EndType to a cosmwasm::timestamp using the corrent block time.
    pub fn set_end_timestamp(&mut self, env: &Env) -> StdResult<ChallengeEntry> {
        let end = match self.end {
            EndType::Duration(DurationChoice::Week) => Duration::weeks(1).to_std().unwrap(),
            EndType::Duration(DurationChoice::Month) => Duration::days(30).to_std().unwrap(),
            EndType::Duration(DurationChoice::Quarter) => Duration::days(90).to_std().unwrap(),
            _ => return Ok(self.clone()),
        };
        Ok(ChallengeEntry {
            name: self.name.clone(),
            collateral: self.collateral.clone(),
            description: self.description.clone(),
            end: EndType::ExactTime(Timestamp::from_seconds(
                env.block.time.seconds() + end.as_secs(),
            )),
            total_check_ins: self.total_check_ins.clone(),
            status: self.status.clone(),
            admin_strikes: self.admin_strikes.clone(),
        })
    }
    /// Returns the timestamp of the end of the challenge or an error if the end is not an ExactTime.
    pub fn get_end_timestamp(&self) -> StdResult<Timestamp> {
        match self.end {
            EndType::ExactTime(time) => Ok(time),
            _ => Err(StdError::generic_err("EndType is not ExactTime")),
        }
    }
    /// Sets the total number of check-ins based on the end time.
    pub fn set_total_check_ins(&mut self, env: &Env) -> StdResult<()> {
        let now = env.block.time;

        // Ensure that end time is in exact time format
        let end_time = match self.get_end_timestamp() {
            Ok(time) => time,
            Err(e) => return Err(e),
        };

        // Calculate the duration in seconds
        let duration_secs = end_time.seconds() - now.seconds();
        let check_in_duration_secs = 86400; // 24 hours in seconds
                                            // Calculate the total number of check-ins
        self.total_check_ins = Some(duration_secs as u128 / check_in_duration_secs as u128);

        Ok(())
    }
}

/// The status of a challenge. This can be used to trigger an automated Croncat job
/// based on the value of the status
#[cosmwasm_schema::cw_serde]
pub enum ChallengeStatus {
    /// The challenge has not been initialized yet. This is the default state.
    Uninitialized,
    /// The challenge is active and can be voted on.
    Active,
    /// The challenge was cancelled and no collateral was paid out.
    Cancelled,
    /// The challenge has pased the end time.
    Over,
}

impl Default for ChallengeStatus {
    fn default() -> Self {
        ChallengeStatus::Uninitialized
    }
}

/// Only this struct and these fields are allowed to be updated.
/// The status cannot be externally updated, it is updated by the contract.
#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntryUpdate {
    pub name: Option<String>,
    pub collateral: Option<OfferAsset>,
    pub description: Option<String>,
    pub end: Option<Timestamp>,
}

#[cosmwasm_schema::cw_serde]
pub enum UpdateFriendsOpKind {
    Add,
    Remove,
}

#[cosmwasm_schema::cw_serde]
pub struct Friend<T: AddressLike> {
    pub address: T,
    pub name: String,
}

impl Friend<String> {
    /// A helper to convert from a string address to an Addr.
    /// Additionally it validates the address.
    pub fn check(self, deps: Deps) -> StdResult<Friend<Addr>> {
        Ok(Friend {
            address: deps.api.addr_validate(&self.address)?,
            name: self.name,
        })
    }
}

#[cosmwasm_schema::cw_serde]
pub struct Vote<T: AddressLike> {
    /// The address of the voter
    pub voter: T,
    /// The vote result
    pub approval: Option<bool>,
    /// Correlates to the last_checked_in field of the CheckIn struct.
    pub for_check_in: Option<Timestamp>,
}

impl Vote<String> {
    /// A helper to convert from a string address to an Addr.
    /// Additionally it validates the address.
    pub fn check(self, deps: Deps) -> StdResult<Vote<Addr>> {
        Ok(Vote {
            voter: deps.api.addr_validate(&self.voter)?,
            approval: self.approval,
            for_check_in: None,
        })
    }
}

impl Vote<Addr> {
    /// If the vote approval field is None, we assume the voter approves,
    /// and return a vote with the approval field set to Some(true).
    pub fn optimisitc(self) -> Vote<Addr> {
        Vote {
            voter: self.voter,
            approval: Some(self.approval.unwrap_or(true)),
            for_check_in: None,
        }
    }
}

/// The check in struct is used to track the admin's check ins.
/// The admin must check in every 24 hours, otherwise they get a strike.
#[cosmwasm_schema::cw_serde]
pub struct CheckIn {
    /// The blockheight of the last check in.
    pub last_checked_in: Timestamp,
    /// The blockheight of the next check in.
    /// In the case of a missed check in, this will always be pushed forward
    /// internally by the contract.
    pub next_check_in_by: Timestamp,
    /// Optional metadata for the check in. For example, a link to a tweet.
    pub metadata: Option<String>,
    /// The vote status of the CheckIn.
    pub status: CheckInStatus,
    /// The final result of the votes for this check in.
    pub tally_result: Option<bool>,
}

#[cosmwasm_schema::cw_serde]
pub enum CheckInStatus {
    /// The admin has not yet checked in, therefore no voting or tallying
    /// has occured for this check in.
    NotCheckedIn,
    /// The admin has checked in, but all friends have not yet all voted.
    /// Some friends may have voted, but not all.
    CheckedInNotYetVoted,
    /// The admin mised their check in and got a strike.
    MissedCheckIn,
    /// The admin vetoed a vote for this checkin, the votes must be recounted.
    Recount,
    /// The admin has checked in and all friends have voted.
    /// But the check in has not yet been tallied.
    VotedNotYetTallied,
    /// The check in has been voted and tallied.
    VotedAndTallied,
}

impl CheckIn {
    pub fn default_from(env: &Env) -> Self {
        CheckIn {
            last_checked_in: Timestamp::from_seconds(env.block.time.seconds()),
            // set the next check in to be 24 hours from now
            next_check_in_by: Timestamp::from_seconds(env.block.time.seconds() + 60 * 60 * 24),
            metadata: None,
            status: CheckInStatus::NotCheckedIn,
            tally_result: None,
        }
    }
}

pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const ADMIN: Item<Addr> = Item::new("admin");
pub const CHALLENGE_LIST: Map<u64, ChallengeEntry> = Map::new("challenge_list");
pub const CHALLENGE_FRIENDS: Map<u64, Vec<Friend<Addr>>> = Map::new("challenge_friends");

/// Key is a tuple of (challenge_id, voter_address).
/// By using a composite key, it ensures only one user can vote per challenge.
pub const VOTES: Map<(u64, Addr), Vote<Addr>> = Map::new("votes");

/// For looking up all the votes by id. This is used to tally the votes.
pub const CHALLENGE_VOTES: Map<u64, Vec<Vote<Addr>>> = Map::new("challenge_votes");

/// For looking up all the check ins by challenge_id.
pub const DAILY_CHECK_INS: Map<u64, Vec<CheckIn>> = Map::new("daily_checkins");
