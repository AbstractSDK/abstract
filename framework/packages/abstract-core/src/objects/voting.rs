use std::fmt::Display;

use cosmwasm_std::{Addr, BlockInfo, Decimal, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use cw_utils::{Duration, Expiration};
use thiserror::Error;

/// Wrapper error for the Abstract framework.
#[derive(Error, Debug, PartialEq)]
pub enum VoteError {
    #[error("Std error encountered while handling voting object: {0}")]
    Std(#[from] StdError),

    #[error("No vote by vote id")]
    NoVoteById {},

    #[error("Actions allowed only for active vote")]
    VoteNotActive {},

    #[error("Threshold error: {0}")]
    ThresholdError(String),

    #[error("Veto period expired, can't cancel vote, expiration: {expiration}")]
    VetoExpired { expiration: Expiration },

    #[error("Only admin can do actions during veto period: {expiration}")]
    VetoNotOver { expiration: Expiration },

    #[error("Veto actions could be done only during veto period, current status: {status}")]
    NotVeto { status: VoteStatus },

    #[error("Vote period expired, can't do actions, expiration: {expiration}")]
    VoteExpired { expiration: Expiration },

    #[error("Too early to count votes: voting is not over")]
    VotingNotOver {},
}

pub type VoteResult<T> = Result<T, VoteError>;

pub const DEFAULT_LIMIT: u64 = 25;
pub type VoteId = u64;

/// Simple voting helper
pub struct SimpleVoting<'a> {
    next_vote_id: Item<'a, VoteId>,
    votes: Map<'a, (VoteId, &'a Addr), Option<Vote>>,
    vote_info: Map<'a, VoteId, VoteInfo>,
    vote_config: Item<'a, VoteConfig>,
}

impl<'a> SimpleVoting<'a> {
    pub const fn new(
        votes_key: &'a str,
        id_key: &'a str,
        vote_info_key: &'a str,
        vote_config_key: &'a str,
    ) -> Self {
        Self {
            next_vote_id: Item::new(id_key),
            votes: Map::new(votes_key),
            vote_info: Map::new(vote_info_key),
            vote_config: Item::new(vote_config_key),
        }
    }

    pub fn instantiate(&self, store: &mut dyn Storage, vote_config: &VoteConfig) -> VoteResult<()> {
        vote_config.threshold.validate_percentage()?;

        self.next_vote_id.save(store, &VoteId::default())?;
        self.vote_config.save(store, vote_config)?;
        Ok(())
    }

    pub fn new_vote(
        &self,
        store: &mut dyn Storage,
        end: Expiration,
        initial_voters: &[Addr],
    ) -> VoteResult<VoteId> {
        let vote_id = self
            .next_vote_id
            .update(store, |id| VoteResult::Ok(id + 1))?;

        self.vote_info.save(
            store,
            vote_id,
            &VoteInfo::new(initial_voters.len() as u32, end),
        )?;
        for voter in initial_voters {
            self.votes.save(store, (vote_id, voter), &None)?;
        }
        Ok(vote_id)
    }

    pub fn cast_vote(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        vote_id: VoteId,
        voter: &Addr,
        vote: Vote,
        // new_voter: bool,
    ) -> VoteResult<VoteInfo> {
        // Need to check it's existing vote
        let mut vote_info = self.load_vote_info(store, vote_id)?;
        vote_info.assert_ready_for_action(block)?;
        // match new_voter {
        //     true => {
        //         if !self.votes.has(store, (vote_id, voter)) {
        //             return Err(crate::AbstractError::Std(StdError::generic_err(
        //                 "New voter is already registered",
        //             )));
        //         }
        //         self.votes.save(store, (vote_id, voter), &Some(vote))?;
        //         // update vote_info
        //         vote_info.total_voters += 1;
        //         self.vote_info.save(store, vote_id, &vote_info)?;
        //     }
        //     false => {
        self.votes.update(
            store,
            (vote_id, voter),
            |previous_vote| match previous_vote {
                // We allow re-voting
                Some(prev_v) => {
                    vote_info.vote_update(prev_v.as_ref(), &vote);
                    Ok(Some(vote))
                }
                None => Err(StdError::generic_err("This user is not allowed to vote")),
            },
        )?;
        // }
        // }
        self.vote_info.save(store, vote_id, &vote_info)?;
        Ok(vote_info)
    }

    // Note: this method doesn't check a sender
    // Therefore caller of this method should check if he is allowed to cancel vote
    pub fn cancel_vote(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        vote_id: VoteId,
    ) -> VoteResult<VoteInfo> {
        let mut vote_info = self.load_vote_info(store, vote_id)?;
        vote_info.assert_ready_for_action(block)?;

        vote_info.status = VoteStatus::Finished(VoteOutcome::Canceled {});
        self.vote_info.save(store, vote_id, &vote_info)?;
        Ok(vote_info)
    }

    pub fn count_votes(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        vote_id: VoteId,
    ) -> VoteResult<VoteInfo> {
        let mut vote_info = self.load_vote_info(store, vote_id)?;
        vote_info.status.assert_is_active()?;
        let vote_config = self.vote_config.load(store)?;

        // Calculate votes
        let threshold = match vote_config.threshold {
            Threshold::Majority {} => Uint128::from(vote_info.total_voters / 2),
            Threshold::Percentage(decimal) => decimal * Uint128::from(vote_info.total_voters),
        };

        let vote_outcome = if Uint128::from(vote_info.votes_for) > threshold {
            VoteOutcome::Passed
        } else if Uint128::from(vote_info.votes_against) > threshold
        // if it's expired or everyone voted it's still should be able to finish voting
            || vote_info.end.is_expired(block)
            || vote_info.votes_against + vote_info.votes_for == vote_info.total_voters
        {
            VoteOutcome::Failed
        } else {
            return Err(VoteError::VotingNotOver {});
        };

        // Update vote status
        vote_info.status = if let Some(duration) = vote_config.veto_duration {
            let veto_exp = duration.after(block);
            VoteStatus::VetoPeriod(veto_exp, vote_outcome)
        } else {
            VoteStatus::Finished(vote_outcome)
        };
        self.vote_info.save(store, vote_id, &vote_info)?;

        Ok(vote_info)
    }

    /// Called by veto admin
    pub fn veto_admin_action(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        vote_id: VoteId,
        admin_action: VetoAdminAction,
    ) -> VoteResult<VoteInfo> {
        let mut vote_info = self.load_vote_info(store, vote_id)?;

        let VoteStatus::VetoPeriod(exp, vote_outcome) = vote_info.status else {
            return Err(VoteError::NotVeto {
                status: vote_info.status,
            });
        };
        if exp.is_expired(block) {
            return Err(VoteError::VetoExpired { expiration: exp });
        }

        vote_info.status = match admin_action {
            VetoAdminAction::Finish {} => VoteStatus::Finished(vote_outcome),
            VetoAdminAction::Veto {} => VoteStatus::Finished(VoteOutcome::Vetoed),
        };
        self.vote_info.save(store, vote_id, &vote_info)?;

        Ok(vote_info)
    }

    /// Called by non-admin
    pub fn finish_vote(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        vote_id: VoteId,
    ) -> VoteResult<VoteInfo> {
        let mut vote_info = self.load_vote_info(store, vote_id)?;

        let VoteStatus::VetoPeriod(expiration, vote_outcome) = vote_info.status else {
            return Err(VoteError::NotVeto {
                status: vote_info.status,
            });
        };
        if !expiration.is_expired(block) {
            return Err(VoteError::VetoNotOver { expiration });
        }

        vote_info.status = VoteStatus::Finished(vote_outcome);
        self.vote_info.save(store, vote_id, &vote_info)?;

        Ok(vote_info)
    }

    pub fn add_voters(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
        block: &BlockInfo,
        new_voters: &[Addr],
    ) -> VoteResult<VoteInfo> {
        // Need to check it's existing vote
        let mut vote_info = self.load_vote_info(store, vote_id)?;
        vote_info.assert_ready_for_action(block)?;

        for voter in new_voters {
            // Don't override already existing vote
            self.votes.update(store, (vote_id, voter), |v| match v {
                Some(v) => VoteResult::Ok(v),
                None => {
                    vote_info.total_voters += 1;
                    VoteResult::Ok(None)
                }
            })?;
        }
        self.vote_info.save(store, vote_id, &vote_info)?;

        Ok(vote_info)
    }

    pub fn remove_voters(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
        block: &BlockInfo,
        removed_voters: &[Addr],
    ) -> VoteResult<VoteInfo> {
        let mut vote_info = self.load_vote_info(store, vote_id)?;
        vote_info.assert_ready_for_action(block)?;

        for voter in removed_voters {
            if let Some(vote) = self.votes.may_load(store, (vote_id, voter))? {
                if let Some(previous_vote) = vote {
                    match previous_vote.vote {
                        true => vote_info.votes_for -= 1,
                        false => vote_info.votes_against -= 1,
                    }
                }
                vote_info.total_voters -= 1;
                self.votes.remove(store, (vote_id, voter));
            }
        }
        self.vote_info.save(store, vote_id, &vote_info)?;
        Ok(vote_info)
    }

    pub fn load_vote(
        &self,
        store: &dyn Storage,
        vote_id: VoteId,
        voter: &Addr,
    ) -> VoteResult<Option<Vote>> {
        self.votes.load(store, (vote_id, voter)).map_err(Into::into)
    }

    pub fn load_vote_info(&self, store: &dyn Storage, vote_id: VoteId) -> VoteResult<VoteInfo> {
        self.vote_info
            .may_load(store, vote_id)?
            .ok_or(VoteError::NoVoteById {})
    }

    pub fn votes_for_id(
        &self,
        store: &dyn Storage,
        vote_id: VoteId,
        start_after: Option<&Addr>,
        limit: Option<u64>,
    ) -> VoteResult<Vec<(Addr, Option<Vote>)>> {
        let min = start_after.map(Bound::exclusive);
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let votes = self
            .votes
            .prefix(vote_id)
            .range(store, min, None, cosmwasm_std::Order::Ascending)
            .take(limit as usize)
            .collect::<StdResult<_>>()?;
        Ok(votes)
    }

    pub fn query_list(
        &self,
        store: &dyn Storage,
        start_after: Option<(VoteId, &Addr)>,
        limit: Option<u64>,
    ) -> VoteResult<Vec<((VoteId, Addr), Option<Vote>)>> {
        let min = start_after.map(Bound::exclusive);
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let votes = self
            .votes
            .range(store, min, None, cosmwasm_std::Order::Ascending)
            .take(limit as usize)
            .collect::<StdResult<_>>()?;
        Ok(votes)
    }
}

/// Vote struct
#[cosmwasm_schema::cw_serde]
pub struct Vote {
    /// true: Vote for
    /// false: Vote against
    pub vote: bool,
    /// memo for the vote
    pub memo: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct VoteInfo {
    pub total_voters: u32,
    pub votes_for: u32,
    pub votes_against: u32,
    pub status: VoteStatus,
    pub end: Expiration,
}

impl VoteInfo {
    pub fn new(initial_voters: u32, end: Expiration) -> Self {
        Self {
            total_voters: initial_voters,
            votes_for: 0,
            votes_against: 0,
            status: VoteStatus::Active {},
            end,
        }
    }

    pub fn assert_ready_for_action(&self, block: &BlockInfo) -> VoteResult<()> {
        self.status.assert_is_active()?;
        if self.end.is_expired(block) {
            Err(VoteError::VoteExpired {
                expiration: self.end,
            })
        } else {
            Ok(())
        }
    }

    pub fn vote_update(&mut self, previous_vote: Option<&Vote>, new_vote: &Vote) {
        match (previous_vote, new_vote.vote) {
            (Some(Vote { vote: true, .. }), true) | (Some(Vote { vote: false, .. }), false) => {}
            (Some(Vote { vote: true, .. }), false) => {
                self.votes_for += 1;
                self.votes_against -= 1;
            }
            (Some(Vote { vote: false, .. }), true) => {
                self.votes_against += 1;
                self.votes_for -= 1;
            }
            (None, true) => {
                self.votes_for += 1;
            }
            (None, false) => {
                self.votes_against += 1;
            }
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub enum VoteStatus {
    Active,
    VetoPeriod(Expiration, VoteOutcome),
    Finished(VoteOutcome),
}

impl Display for VoteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoteStatus::Active => write!(f, "active"),
            VoteStatus::VetoPeriod(exp, outcome) => write!(f, "veto_period({exp},{outcome})"),
            VoteStatus::Finished(outcome) => write!(f, "finished({outcome})"),
        }
    }
}

impl VoteStatus {
    pub fn assert_is_active(&self) -> VoteResult<()> {
        match self {
            VoteStatus::Active => Ok(()),
            _ => Err(VoteError::VoteNotActive {}),
        }
    }
}

// TODO: do we want more info like BlockInfo?
#[cosmwasm_schema::cw_serde]
pub enum VoteOutcome {
    Passed,
    Failed,
    Canceled,
    Vetoed,
}

impl Display for VoteOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoteOutcome::Passed => write!(f, "passed"),
            VoteOutcome::Failed => write!(f, "failed"),
            VoteOutcome::Canceled => write!(f, "canceled"),
            VoteOutcome::Vetoed => write!(f, "vetoed"),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct VoteConfig {
    pub threshold: Threshold,
    /// Veto duration after the first vote
    /// None disables veto
    pub veto_duration: Option<Duration>,
}

#[cosmwasm_schema::cw_serde]
pub enum Threshold {
    Majority {},
    Percentage(Decimal),
}

impl Threshold {
    /// Asserts that the 0.0 < percent <= 1.0
    fn validate_percentage(&self) -> VoteResult<()> {
        if let Threshold::Percentage(percent) = self {
            if percent.is_zero() {
                Err(VoteError::ThresholdError("can't be 0%".to_owned()))
            } else if *percent > Decimal::one() {
                Err(VoteError::ThresholdError(
                    "not possible to reach >100% votes".to_owned(),
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

/// Admin actions on veto
#[cosmwasm_schema::cw_serde]
pub enum VetoAdminAction {
    /// Fast-forward vote
    Finish {},
    /// Veto this vote
    Veto {},
}
