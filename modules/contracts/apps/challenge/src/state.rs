use abstract_dex_adapter::msg::OfferAsset;
use chrono::Duration;
use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Timestamp};
use cw_address_like::AddressLike;
use cw_storage_plus::{Item, Map};

use crate::msg::{ChallengeRequest, DurationChoice};

pub const DAY: u64 = 86400;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntry {
    pub name: String,
    pub collateral: OfferAsset,
    pub description: String,
    pub end: Timestamp,
    pub total_check_ins: u128,
    pub status: ChallengeStatus,
    pub admin_strikes: StrikeConfig,
}

#[cosmwasm_schema::cw_serde]
pub struct StrikeConfig {
    /// The number of striked the admin has incurred.
    pub num_strikes: u8,
    /// When num_strikes reached the limit, the challenge will be cancelled.
    pub limit: u8,
}

impl Default for StrikeConfig {
    fn default() -> Self {
        StrikeConfig {
            num_strikes: 0,
            limit: 3,
        }
    }
}

impl ChallengeEntry {
    /// Creates a new challenge entry with the default status of Uninitialized and no admin strikes.
    pub fn new(request: ChallengeRequest) -> Self {
        ChallengeEntry {
            name: request.name,
            collateral: request.collateral,
            description: request.description,
            end: Self::to_timestamp(request.end),
            total_check_ins: 0,
            status: ChallengeStatus::default(),
            admin_strikes: StrikeConfig::default(),
        }
    }

    /// Sets the total number of check-ins based on the end time.
    pub fn set_total_check_ins(&mut self, env: &Env) -> StdResult<()> {
        let now = env.block.time;

        match self.end.seconds().checked_sub(now.seconds()) {
            Some(duration_secs) => {
                // Calculate the total number of check-ins
                self.total_check_ins = duration_secs as u128 / DAY as u128;
                Ok(())
            }
            None => {
                // If the end time is in the past, set the total check ins to 0.
                self.total_check_ins = 0;
                // If the end time is in the past or there's an overflow in calculation, return an error.
                Err(StdError::generic_err(format!(
                    "Cannot compute. challenge.end time is in the past: {}, now is: {}",
                    self.end, now
                )))
            }
        }
    }

    pub fn to_timestamp(end: DurationChoice) -> Timestamp {
        match end {
            DurationChoice::Week => {
                Timestamp::from_seconds(Duration::weeks(1).to_std().unwrap().as_secs())
            }
            DurationChoice::Month => {
                Timestamp::from_seconds(Duration::days(30).to_std().unwrap().as_secs())
            }
            DurationChoice::Quarter => {
                Timestamp::from_seconds(Duration::days(90).to_std().unwrap().as_secs())
            }
            DurationChoice::Year => {
                Timestamp::from_seconds(Duration::days(365).to_std().unwrap().as_secs())
            }
            DurationChoice::OneHundredYears => {
                Timestamp::from_seconds(Duration::days(365 * 100).to_std().unwrap().as_secs())
            }
        }
    }
}

/// The status of a challenge. This can be used to trigger an automated Croncat job
/// based on the value of the status
#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub enum ChallengeStatus {
    /// The challenge has not been initialized yet. This is the default state.
    #[default]
    Uninitialized,
    /// The challenge is active and can be voted on.
    Active,
    /// The challenge was cancelled and no collateral was paid out.
    Cancelled,
    /// The challenge has pased the end time.
    Over,
}

/// Only this struct and these fields are allowed to be updated.
/// The status cannot be externally updated, it is updated by the contract.
#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntryUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
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
    pub fn optimistic(self) -> Vote<Addr> {
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
    pub last: Timestamp,
    /// The blockheight of the next check in.
    /// In the case of a missed check in, this will always be pushed forward
    /// internally by the contract.
    pub next: Timestamp,
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
    /// The admin has checked in and all friends have voted.
    /// But the check in has not yet been tallied.
    VotedNotYetTallied,
    /// The check in has been voted and tallied.
    VotedAndTallied,
}

impl CheckIn {
    pub fn default_from(env: &Env) -> Self {
        CheckIn {
            last: Timestamp::from_seconds(env.block.time.seconds()),
            // set the next check in to be 24 hours from now
            next: Timestamp::from_seconds(env.block.time.seconds() + 60 * 60 * 24),
            metadata: None,
            status: CheckInStatus::NotCheckedIn,
            tally_result: None,
        }
    }
}

pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const CHALLENGE_LIST: Map<u64, ChallengeEntry> = Map::new("challenge_list");
pub const CHALLENGE_FRIENDS: Map<u64, Vec<Friend<Addr>>> = Map::new("challenge_friends");

/// Key is a tuple of (challenge_id, check_in.last_checked_in, voter_address).
/// By using a composite key, it ensures only one user can vote per check_in.
pub const VOTES: Map<(u64, u64, Addr), Vote<Addr>> = Map::new("votes");

/// For looking up all the votes by challenge_id. This is used to tally the votes.
pub const CHALLENGE_VOTES: Map<u64, Vec<Vote<Addr>>> = Map::new("challenge_votes");

/// For looking up all the check ins for a challenge_id.
pub const DAILY_CHECK_INS: Map<u64, Vec<CheckIn>> = Map::new("daily_checkins");
