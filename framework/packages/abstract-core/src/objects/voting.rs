//! # Simple voting
//! Simple voting is a state object to enable voting mechanism on a contract
//!
//! ## Setting up
//! * Create SimpleVoting object in similar way to the cw-storage-plus objects using [`SimpleVoting::new`] method
//! * Inside instantiate contract method use [`SimpleVoting::instantiate`] method
//! * Add [`VoteError`] type to your application errors
//!
//! ## Creating a new proposal
//! To create a new proposal use [`SimpleVoting::new_proposal`] method, it will return ProposalId
//!
//! ## Whitelisting voters
//! Initial whitelist passed during [`SimpleVoting::new_proposal`] method, you can use
//! - [`SimpleVoting::add_voters`] to whitelist new voters
//! - [`SimpleVoting::remove_voters`] to remove voters(and their votes) from whitelist
//!
//! ## Voting
//! To cast a vote use [`SimpleVoting::cast_vote`] method
//!
//! ## Count voting
//! To count votes use [`SimpleVoting::count_votes`] method
//!
//! ## Veto action
//! In case your [`VoteConfig`] has veto duration set-up, after successful vote-count veto period will start
//! * During veto period [`SimpleVoting::veto_admin_action`] method could be used to finish(fast-forward) or Veto proposal
//! * After veto period [`SimpleVoting::finish_vote`] method could be used to finish proposal
//!
//! ## Cancel proposal
//! During active voting(before veto or finishing vote),
//! [`SimpleVoting::cancel_proposal`] method could be used to cancel proposal
//!
//! ## Queries
//! * Single-item queries methods allowed by `load_` prefix
//! * List of items queries allowed by `query_` prefix
//!
//! ## Details
//! All methods that modify proposal will return [`ProposalInfo`] to allow logging or checking current status of proposal

use std::fmt::Display;

use cosmwasm_std::{Addr, BlockInfo, Decimal, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use cw_utils::{Duration, Expiration};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VoteError {
    #[error("Std error encountered while handling voting object: {0}")]
    Std(#[from] StdError),

    #[error("No proposal by proposal id")]
    NoProposalById {},

    #[error("Actions allowed only for active proposal")]
    ProposalNotActive {},

    #[error("Threshold error: {0}")]
    ThresholdError(String),

    #[error("Veto period expired, can't cancel proposal, expiration: {expiration}")]
    VetoExpired { expiration: Expiration },

    #[error("Only admin can do actions during veto period: {expiration}")]
    VetoNotOver { expiration: Expiration },

    #[error("Veto actions could be done only during veto period, current status: {status}")]
    NotVeto { status: ProposalStatus },

    #[error("Proposal period expired, can't do actions, expiration: {expiration}")]
    ProposalExpired { expiration: Expiration },

    #[error("Too early to count votes: voting is not over")]
    VotingNotOver {},
}

pub type VoteResult<T> = Result<T, VoteError>;

pub const DEFAULT_LIMIT: u64 = 25;
pub type ProposalId = u64;

/// Simple voting helper
pub struct SimpleVoting<'a> {
    next_proposal_id: Item<'a, ProposalId>,
    proposals: Map<'a, (ProposalId, &'a Addr), Option<Vote>>,
    proposals_info: Map<'a, ProposalId, ProposalInfo>,
    vote_config: Item<'a, VoteConfig>,
}

impl<'a> SimpleVoting<'a> {
    pub const fn new(
        proposals_key: &'a str,
        id_key: &'a str,
        proposals_info_key: &'a str,
        vote_config_key: &'a str,
    ) -> Self {
        Self {
            next_proposal_id: Item::new(id_key),
            proposals: Map::new(proposals_key),
            proposals_info: Map::new(proposals_info_key),
            vote_config: Item::new(vote_config_key),
        }
    }

    /// SimpleVoting setup during instantiation
    pub fn instantiate(&self, store: &mut dyn Storage, vote_config: &VoteConfig) -> VoteResult<()> {
        vote_config.threshold.validate_percentage()?;

        self.next_proposal_id.save(store, &ProposalId::default())?;
        self.vote_config.save(store, vote_config)?;
        Ok(())
    }

    pub fn update_vote_config(
        &self,
        store: &mut dyn Storage,
        new_vote_config: &VoteConfig,
    ) -> StdResult<()> {
        self.vote_config.save(store, new_vote_config)
    }

    /// Create new proposal
    /// initial_voters is a list of whitelisted to vote
    pub fn new_proposal(
        &self,
        store: &mut dyn Storage,
        end: Expiration,
        initial_voters: &[Addr],
    ) -> VoteResult<ProposalId> {
        let proposal_id = self
            .next_proposal_id
            .update(store, |id| VoteResult::Ok(id + 1))?;

        self.proposals_info.save(
            store,
            proposal_id,
            &ProposalInfo::new(initial_voters.len() as u32, end),
        )?;
        for voter in initial_voters {
            self.proposals.save(store, (proposal_id, voter), &None)?;
        }
        Ok(proposal_id)
    }

    /// Assign vote for the voter
    pub fn cast_vote(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        proposal_id: ProposalId,
        voter: &Addr,
        vote: Vote,
        // new_voter: bool,
    ) -> VoteResult<ProposalInfo> {
        // Need to check it's existing proposal
        let mut proposal_info = self.load_proposal(store, proposal_id)?;
        proposal_info.assert_ready_for_action(block)?;
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
        self.proposals.update(
            store,
            (proposal_id, voter),
            |previous_vote| match previous_vote {
                // We allow re-voting
                Some(prev_v) => {
                    proposal_info.vote_update(prev_v.as_ref(), &vote);
                    Ok(Some(vote))
                }
                None => Err(StdError::generic_err("This user is not allowed to vote")),
            },
        )?;
        // }
        // }
        self.proposals_info
            .save(store, proposal_id, &proposal_info)?;
        Ok(proposal_info)
    }

    // Note: this method doesn't check a sender
    // Therefore caller of this method should check if he is allowed to cancel vote
    /// Cancel proposal
    pub fn cancel_proposal(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        proposal_id: ProposalId,
    ) -> VoteResult<ProposalInfo> {
        let mut proposal_info = self.load_proposal(store, proposal_id)?;
        proposal_info.assert_ready_for_action(block)?;

        proposal_info.status = ProposalStatus::Finished(ProposalOutcome::Canceled {});
        self.proposals_info
            .save(store, proposal_id, &proposal_info)?;
        Ok(proposal_info)
    }

    /// Count votes and finish or move to the veto period(if configured) for this proposal
    pub fn count_votes(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        proposal_id: ProposalId,
    ) -> VoteResult<ProposalInfo> {
        let mut proposal_info = self.load_proposal(store, proposal_id)?;
        proposal_info.status.assert_is_active()?;
        let vote_config = self.vote_config.load(store)?;

        // Calculate votes
        let threshold = match vote_config.threshold {
            Threshold::Majority {} => Uint128::from(proposal_info.total_voters / 2),
            Threshold::Percentage(decimal) => decimal * Uint128::from(proposal_info.total_voters),
        };
        // 90% threshold
        // 10% against should be enough to fail the vote
        let reverse_threshold = Uint128::from(proposal_info.total_voters) - threshold;

        let proposal_outcome = if Uint128::from(proposal_info.votes_for) > threshold {
            ProposalOutcome::Passed
        } else if Uint128::from(proposal_info.votes_against) >= reverse_threshold
            || proposal_info.end.is_expired(block)
        {
            ProposalOutcome::Failed
        } else {
            return Err(VoteError::VotingNotOver {});
        };

        // Update vote status
        if let Some(duration) = vote_config.veto_duration {
            let veto_exp = duration.after(block);
            proposal_info.status = ProposalStatus::VetoPeriod(veto_exp, proposal_outcome);
        } else {
            proposal_info.finish_vote(proposal_outcome, block);
        };
        self.proposals_info
            .save(store, proposal_id, &proposal_info)?;

        Ok(proposal_info)
    }

    /// Called by veto admin
    /// Finish or Veto this proposal
    pub fn veto_admin_action(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        proposal_id: ProposalId,
        admin_action: VetoAdminAction,
    ) -> VoteResult<ProposalInfo> {
        let mut proposal_info = self.load_proposal(store, proposal_id)?;

        let ProposalStatus::VetoPeriod(exp, proposal_outcome) = proposal_info.status else {
            return Err(VoteError::NotVeto {
                status: proposal_info.status,
            });
        };
        if exp.is_expired(block) {
            return Err(VoteError::VetoExpired { expiration: exp });
        }

        proposal_info.status = match admin_action {
            VetoAdminAction::Finish {} => ProposalStatus::Finished(proposal_outcome),
            VetoAdminAction::Veto {} => ProposalStatus::Finished(ProposalOutcome::Vetoed),
        };
        println!("got here lol");
        self.proposals_info
            .save(store, proposal_id, &proposal_info)?;

        Ok(proposal_info)
    }

    /// Finish expired veto
    /// Called by non-admin
    pub fn finish_vote(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        proposal_id: ProposalId,
    ) -> VoteResult<ProposalInfo> {
        let mut proposal_info = self.load_proposal(store, proposal_id)?;

        let ProposalStatus::VetoPeriod(expiration, proposal_outcome) = proposal_info.status else {
            return Err(VoteError::NotVeto {
                status: proposal_info.status,
            });
        };
        if !expiration.is_expired(block) {
            return Err(VoteError::VetoNotOver { expiration });
        }

        proposal_info.status = ProposalStatus::VetoPeriod(expiration, proposal_outcome.clone());
        proposal_info.finish_vote(proposal_outcome, block);
        self.proposals_info
            .save(store, proposal_id, &proposal_info)?;

        Ok(proposal_info)
    }

    /// Add new addresses that's allowed to vote
    pub fn add_voters(
        &self,
        store: &mut dyn Storage,
        proposal_id: ProposalId,
        block: &BlockInfo,
        new_voters: &[Addr],
    ) -> VoteResult<ProposalInfo> {
        // Need to check it's existing proposal
        let mut proposal_info = self.load_proposal(store, proposal_id)?;
        proposal_info.assert_ready_for_action(block)?;

        for voter in new_voters {
            // Don't override already existing vote
            self.proposals
                .update(store, (proposal_id, voter), |v| match v {
                    Some(v) => VoteResult::Ok(v),
                    None => {
                        proposal_info.total_voters += 1;
                        VoteResult::Ok(None)
                    }
                })?;
        }
        self.proposals_info
            .save(store, proposal_id, &proposal_info)?;

        Ok(proposal_info)
    }

    /// Remove addresses that's allowed to vote
    /// Will re-count votes
    pub fn remove_voters(
        &self,
        store: &mut dyn Storage,
        proposal_id: ProposalId,
        block: &BlockInfo,
        removed_voters: &[Addr],
    ) -> VoteResult<ProposalInfo> {
        let mut proposal_info = self.load_proposal(store, proposal_id)?;
        proposal_info.assert_ready_for_action(block)?;

        for voter in removed_voters {
            if let Some(vote) = self.proposals.may_load(store, (proposal_id, voter))? {
                if let Some(previous_vote) = vote {
                    match previous_vote.vote {
                        true => proposal_info.votes_for -= 1,
                        false => proposal_info.votes_against -= 1,
                    }
                }
                proposal_info.total_voters -= 1;
                self.proposals.remove(store, (proposal_id, voter));
            }
        }
        self.proposals_info
            .save(store, proposal_id, &proposal_info)?;
        Ok(proposal_info)
    }

    /// Load vote by address
    pub fn load_vote(
        &self,
        store: &dyn Storage,
        proposal_id: ProposalId,
        voter: &Addr,
    ) -> VoteResult<Option<Vote>> {
        self.proposals
            .load(store, (proposal_id, voter))
            .map_err(Into::into)
    }

    /// Load proposal by id
    pub fn load_proposal(
        &self,
        store: &dyn Storage,
        proposal_id: ProposalId,
    ) -> VoteResult<ProposalInfo> {
        self.proposals_info
            .may_load(store, proposal_id)?
            .ok_or(VoteError::NoProposalById {})
    }

    /// List of votes by proposal id
    pub fn query_by_id(
        &self,
        store: &dyn Storage,
        proposal_id: ProposalId,
        start_after: Option<&Addr>,
        limit: Option<u64>,
    ) -> VoteResult<Vec<(Addr, Option<Vote>)>> {
        let min = start_after.map(Bound::exclusive);
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let votes = self
            .proposals
            .prefix(proposal_id)
            .range(store, min, None, cosmwasm_std::Order::Ascending)
            .take(limit as usize)
            .collect::<StdResult<_>>()?;
        Ok(votes)
    }

    pub fn query_list(
        &self,
        store: &dyn Storage,
        start_after: Option<(ProposalId, &Addr)>,
        limit: Option<u64>,
    ) -> VoteResult<Vec<((ProposalId, Addr), Option<Vote>)>> {
        let min = start_after.map(Bound::exclusive);
        let limit = limit.unwrap_or(DEFAULT_LIMIT);

        let votes = self
            .proposals
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
pub struct ProposalInfo {
    pub total_voters: u32,
    pub votes_for: u32,
    pub votes_against: u32,
    pub status: ProposalStatus,
    pub end: Expiration,
}

impl ProposalInfo {
    pub fn new(initial_voters: u32, end: Expiration) -> Self {
        Self {
            total_voters: initial_voters,
            votes_for: 0,
            votes_against: 0,
            status: ProposalStatus::Active {},
            end,
        }
    }

    pub fn assert_ready_for_action(&self, block: &BlockInfo) -> VoteResult<()> {
        self.status.assert_is_active()?;
        if self.end.is_expired(block) {
            Err(VoteError::ProposalExpired {
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

    pub fn finish_vote(&mut self, outcome: ProposalOutcome, block: &BlockInfo) {
        self.status = ProposalStatus::Finished(outcome);
        self.end = match self.end {
            Expiration::AtHeight(_) => Expiration::AtHeight(block.height),
            Expiration::AtTime(_) => Expiration::AtTime(block.time.clone()),
            Expiration::Never {} => Expiration::Never {},
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub enum ProposalStatus {
    Active,
    VetoPeriod(Expiration, ProposalOutcome),
    Finished(ProposalOutcome),
}

impl Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Active => write!(f, "active"),
            ProposalStatus::VetoPeriod(exp, outcome) => write!(f, "veto_period({exp},{outcome})"),
            ProposalStatus::Finished(outcome) => write!(f, "finished({outcome})"),
        }
    }
}

impl ProposalStatus {
    pub fn assert_is_active(&self) -> VoteResult<()> {
        match self {
            ProposalStatus::Active => Ok(()),
            _ => Err(VoteError::ProposalNotActive {}),
        }
    }
}

// TODO: do we want more info like BlockInfo?
#[cosmwasm_schema::cw_serde]
pub enum ProposalOutcome {
    Passed,
    Failed,
    Canceled,
    Vetoed,
}

impl Display for ProposalOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalOutcome::Passed => write!(f, "passed"),
            ProposalOutcome::Failed => write!(f, "failed"),
            ProposalOutcome::Canceled => write!(f, "canceled"),
            ProposalOutcome::Vetoed => write!(f, "vetoed"),
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
