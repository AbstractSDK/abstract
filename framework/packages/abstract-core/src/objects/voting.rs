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

use std::{collections::HashSet, fmt::Display};

use cosmwasm_std::{
    Addr, BlockInfo, Decimal, StdError, StdResult, Storage, Timestamp, Uint128, Uint64,
};
use cw_storage_plus::{Bound, Item, Map};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VoteError {
    #[error("Std error encountered while handling voting object: {0}")]
    Std(#[from] StdError),

    #[error("Tried to add duplicate voter addresses")]
    DuplicateAddrs {},

    #[error("No proposal by proposal id")]
    NoProposalById {},

    #[error("Actions allowed only for active proposal")]
    ProposalNotActive {},

    #[error("Threshold error: {0}")]
    ThresholdError(String),

    #[error("Veto period expired, can't cancel proposal, expiration: {expiration}")]
    VetoExpired { expiration: Timestamp },

    #[error("Only admin can do actions during veto period: {expiration}")]
    VetoNotOver { expiration: Timestamp },

    #[error("Veto actions could be done only during veto period, current status: {status}")]
    NotVeto { status: ProposalStatus },

    #[error("Proposal period expired, can't do actions, expiration: {expiration}")]
    ProposalExpired { expiration: Timestamp },

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
        end: Timestamp,
        initial_voters: &[Addr],
    ) -> VoteResult<ProposalId> {
        // Check if addrs unique
        let mut unique_addrs = HashSet::with_capacity(initial_voters.len());
        if !initial_voters.iter().all(|x| unique_addrs.insert(x)) {
            return Err(VoteError::DuplicateAddrs {});
        }

        let proposal_id = self
            .next_proposal_id
            .update(store, |id| VoteResult::Ok(id + 1))?;

        let config = self.load_config(store)?;
        self.proposals_info.save(
            store,
            proposal_id,
            &ProposalInfo::new(initial_voters.len() as u32, config, end),
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
    ) -> VoteResult<ProposalInfo> {
        let mut proposal_info = self.load_proposal(store, block, proposal_id)?;
        proposal_info.assert_active_proposal(block)?;

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
        let mut proposal_info = self.load_proposal(store, block, proposal_id)?;
        proposal_info.assert_active_proposal(block)?;

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
    ) -> VoteResult<(ProposalInfo, ProposalOutcome)> {
        let mut proposal_info = self.load_proposal(store, block, proposal_id)?;
        let end = match proposal_info.config.veto_duration_seconds {
            Some(dur) => proposal_info.end_timestamp.plus_seconds(dur.u64()),
            None => proposal_info.end_timestamp,
        };
        // check if voting over
        if block.time < end {
            return Err(VoteError::VotingNotOver {});
        }
        proposal_info.status.assert_is_active()?;
        let vote_config = &proposal_info.config;

        // Calculate votes
        let threshold = match vote_config.threshold {
            Threshold::Majority {} => Uint128::from(proposal_info.total_voters / 2),
            Threshold::Percentage(decimal) => decimal * Uint128::from(proposal_info.total_voters),
        };

        let proposal_outcome = if Uint128::from(proposal_info.votes_for) > threshold {
            ProposalOutcome::Passed
        } else {
            ProposalOutcome::Failed
        };

        // Update vote status
        proposal_info.finish_vote(proposal_outcome, block);
        self.proposals_info
            .save(store, proposal_id, &proposal_info)?;

        Ok((proposal_info, proposal_outcome))
    }

    /// Called by veto admin
    /// Finish or Veto this proposal
    pub fn veto_proposal(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        proposal_id: ProposalId,
    ) -> VoteResult<ProposalInfo> {
        let mut proposal_info = self.load_proposal(store, block, proposal_id)?;

        let ProposalStatus::VetoPeriod(_) = proposal_info.status else {
            return Err(VoteError::NotVeto {
                status: proposal_info.status,
            });
        };

        proposal_info.status = ProposalStatus::Finished(ProposalOutcome::Vetoed);
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
        let mut proposal_info = self.load_proposal(store, block, proposal_id)?;
        proposal_info.assert_active_proposal(block)?;

        for voter in new_voters {
            // Don't override already existing vote
            self.proposals
                .update(store, (proposal_id, voter), |v| match v {
                    Some(_) => Err(VoteError::DuplicateAddrs {}),
                    None => {
                        proposal_info.total_voters += 1;
                        Ok(None)
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
        let mut proposal_info = self.load_proposal(store, block, proposal_id)?;
        proposal_info.assert_active_proposal(block)?;

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
        block: &BlockInfo,
        proposal_id: ProposalId,
    ) -> VoteResult<ProposalInfo> {
        let mut proposal = self
            .proposals_info
            .may_load(store, proposal_id)?
            .ok_or(VoteError::NoProposalById {})?;
        // Check if veto period and update if so
        if let Some(veto_duration) = proposal.config.veto_duration_seconds {
            let veto_expiration = proposal.end_timestamp.plus_seconds(veto_duration.u64());
            if block.time > proposal.end_timestamp && block.time <= veto_expiration {
                proposal.status = ProposalStatus::VetoPeriod(veto_expiration)
            }
        }
        Ok(proposal)
    }

    /// Load current vote config
    pub fn load_config(&self, store: &dyn Storage) -> StdResult<VoteConfig> {
        self.vote_config.load(store)
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

    #[allow(clippy::type_complexity)]
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
    /// Config it was created with
    /// For cases config got changed during voting
    pub config: VoteConfig,
    pub end_timestamp: Timestamp,
}

impl ProposalInfo {
    pub fn new(initial_voters: u32, config: VoteConfig, end_timestamp: Timestamp) -> Self {
        Self {
            total_voters: initial_voters,
            votes_for: 0,
            votes_against: 0,
            config,
            status: ProposalStatus::Active {},
            end_timestamp,
        }
    }

    pub fn assert_active_proposal(&self, block: &BlockInfo) -> VoteResult<()> {
        self.status.assert_is_active()?;
        if block.time > self.end_timestamp {
            Err(VoteError::ProposalExpired {
                expiration: self.end_timestamp,
            })
        } else {
            Ok(())
        }
    }

    pub fn vote_update(&mut self, previous_vote: Option<&Vote>, new_vote: &Vote) {
        match (previous_vote, new_vote.vote) {
            // unchanged vote
            (Some(Vote { vote: true, .. }), true) | (Some(Vote { vote: false, .. }), false) => {}
            // vote for became vote against
            (Some(Vote { vote: true, .. }), false) => {
                self.votes_against += 1;
                self.votes_for -= 1;
            }
            // vote against became vote for
            (Some(Vote { vote: false, .. }), true) => {
                self.votes_for += 1;
                self.votes_against -= 1;
            }
            // new vote for
            (None, true) => {
                self.votes_for += 1;
            }
            // new vote against
            (None, false) => {
                self.votes_against += 1;
            }
        }
    }

    pub fn finish_vote(&mut self, outcome: ProposalOutcome, block: &BlockInfo) {
        self.status = ProposalStatus::Finished(outcome);
        self.end_timestamp = block.time
    }
}

#[cosmwasm_schema::cw_serde]
pub enum ProposalStatus {
    Active,
    VetoPeriod(Timestamp),
    Finished(ProposalOutcome),
}

impl Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Active => write!(f, "active"),
            ProposalStatus::VetoPeriod(exp) => write!(f, "veto_period until {exp}"),
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

#[cosmwasm_schema::cw_serde]
#[derive(Copy)]
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
    pub veto_duration_seconds: Option<Uint64>,
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
