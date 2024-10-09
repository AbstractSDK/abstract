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
//! Initial whitelist passed during [`SimpleVoting::new_proposal`] method and currently has no way to edit this
//!
//! ## Voting
//! To cast a vote use [`SimpleVoting::cast_vote`] method
//!
//! ## Count voting
//! To count votes use [`SimpleVoting::count_votes`] method during [`ProposalStatus::WaitingForCount`]
//!
//! ## Veto
//! In case your [`VoteConfig`] has veto duration set-up, after proposal.end_timestamp veto period will start
//! * During veto period [`SimpleVoting::veto_proposal`] method could be used to Veto proposal
//!
//! ## Cancel proposal
//! During active voting:
//! * [`SimpleVoting::cancel_proposal`] method could be used to cancel proposal
//!
//! ## Queries
//! * Single-item queries methods allowed by `load_` prefix
//! * List of items queries allowed by `query_` prefix
//!
//! ## Details
//! All methods that modify proposal will return [`ProposalInfo`] to allow logging or checking current status of proposal.
//!
//! Each proposal goes through the following stages:
//! 1. Active: proposal is active and can be voted on. It can also be canceled during this period.
//! 3. VetoPeriod (optional): voting is counted and veto period is active.
//! 2. WaitingForCount: voting period is finished and awaiting counting.
//! 4. Finished: proposal is finished and count is done. The proposal then has one of the following end states:
//!     * Passed: proposal passed
//!     * Failed: proposal failed
//!     * Canceled: proposal was canceled
//!     * Vetoed: proposal was vetoed

use std::{collections::HashSet, fmt::Display};

use cosmwasm_std::{
    ensure_eq, Addr, BlockInfo, Decimal, StdError, StdResult, Storage, Timestamp, Uint128, Uint64,
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

    #[error("Action allowed only for active proposal")]
    ProposalNotActive(ProposalStatus),

    #[error("Threshold error: {0}")]
    ThresholdError(String),

    #[error("Veto actions could be done only during veto period, current status: {status}")]
    NotVeto { status: ProposalStatus },

    #[error("Too early to count votes: voting is not over")]
    VotingNotOver {},

    #[error("User is not allowed to vote on this proposal")]
    Unauthorized {},
}

pub type VoteResult<T> = Result<T, VoteError>;

pub const DEFAULT_LIMIT: u64 = 25;
pub type ProposalId = u64;

/// Simple voting helper
pub struct SimpleVoting<'a> {
    next_proposal_id: Item<ProposalId>,
    proposals: Map<(ProposalId, &'a Addr), Option<Vote>>,
    proposals_info: Map<ProposalId, ProposalInfo>,
    vote_config: Item<VoteConfig>,
}

impl<'a> SimpleVoting<'a> {
    pub const fn new(
        proposals_key: &'static str,
        id_key: &'static str,
        proposals_info_key: &'static str,
        vote_config_key: &'static str,
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
        proposal_info.assert_active_proposal()?;

        self.proposals.update(
            store,
            (proposal_id, voter),
            |previous_vote| match previous_vote {
                // We allow re-voting
                Some(prev_v) => {
                    proposal_info.vote_update(prev_v.as_ref(), &vote);
                    Ok(Some(vote))
                }
                None => Err(VoteError::Unauthorized {}),
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
        proposal_info.assert_active_proposal()?;

        proposal_info.finish_vote(ProposalOutcome::Canceled {}, block);
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
        ensure_eq!(
            proposal_info.status,
            ProposalStatus::WaitingForCount,
            VoteError::VotingNotOver {}
        );

        let vote_config = &proposal_info.config;

        // Calculate votes
        let threshold = match vote_config.threshold {
            // 50% + 1 voter
            Threshold::Majority {} => Uint128::from(proposal_info.total_voters / 2 + 1),
            Threshold::Percentage(decimal) => {
                Uint128::from(proposal_info.total_voters).mul_floor(decimal)
            }
        };

        let proposal_outcome = if Uint128::from(proposal_info.votes_for) >= threshold {
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

    /// Load proposal by id with updated status if required
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
        if let ProposalStatus::Active = proposal.status {
            let veto_expiration = proposal.end_timestamp.plus_seconds(
                proposal
                    .config
                    .veto_duration_seconds
                    .unwrap_or_default()
                    .u64(),
            );
            // Check if veto or count period and update if so
            if block.time >= proposal.end_timestamp {
                if block.time < veto_expiration {
                    proposal.status = ProposalStatus::VetoPeriod(veto_expiration)
                } else {
                    proposal.status = ProposalStatus::WaitingForCount
                }
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

    pub fn assert_active_proposal(&self) -> VoteResult<()> {
        self.status.assert_is_active()
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
    WaitingForCount,
    Finished(ProposalOutcome),
}

impl Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Active => write!(f, "active"),
            ProposalStatus::VetoPeriod(exp) => write!(f, "veto_period until {exp}"),
            ProposalStatus::WaitingForCount => write!(f, "waiting_for_count"),
            ProposalStatus::Finished(outcome) => write!(f, "finished({outcome})"),
        }
    }
}

impl ProposalStatus {
    pub fn assert_is_active(&self) -> VoteResult<()> {
        match self {
            ProposalStatus::Active => Ok(()),
            _ => Err(VoteError::ProposalNotActive(self.clone())),
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

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    use super::*;
    const SIMPLE_VOTING: SimpleVoting =
        SimpleVoting::new("proposals", "id", "proposal_info", "config");

    fn setup(storage: &mut dyn Storage, vote_config: &VoteConfig) {
        SIMPLE_VOTING.instantiate(storage, vote_config).unwrap();
    }
    fn default_setup(storage: &mut dyn Storage) {
        setup(
            storage,
            &VoteConfig {
                threshold: Threshold::Majority {},
                veto_duration_seconds: None,
            },
        );
    }

    #[test]
    fn threshold_validation() {
        assert!(Threshold::Majority {}.validate_percentage().is_ok());
        assert!(Threshold::Percentage(Decimal::one())
            .validate_percentage()
            .is_ok());
        assert!(Threshold::Percentage(Decimal::percent(1))
            .validate_percentage()
            .is_ok());

        assert_eq!(
            Threshold::Percentage(Decimal::percent(101)).validate_percentage(),
            Err(VoteError::ThresholdError(
                "not possible to reach >100% votes".to_owned()
            ))
        );
        assert_eq!(
            Threshold::Percentage(Decimal::zero()).validate_percentage(),
            Err(VoteError::ThresholdError("can't be 0%".to_owned()))
        );
    }

    #[test]
    fn assert_active_proposal() {
        let end_timestamp = Timestamp::from_seconds(100);

        // Normal proposal
        let mut proposal = ProposalInfo {
            total_voters: 2,
            votes_for: 0,
            votes_against: 0,
            status: ProposalStatus::Active,
            config: VoteConfig {
                threshold: Threshold::Majority {},
                veto_duration_seconds: Some(Uint64::new(10)),
            },
            end_timestamp,
        };
        assert!(proposal.assert_active_proposal().is_ok());

        // Not active
        proposal.status = ProposalStatus::VetoPeriod(end_timestamp.plus_seconds(10));
        assert_eq!(
            proposal.assert_active_proposal().unwrap_err(),
            VoteError::ProposalNotActive(ProposalStatus::VetoPeriod(
                end_timestamp.plus_seconds(10)
            ))
        );
    }

    #[test]
    fn create_proposal() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let storage = &mut deps.storage;
        default_setup(storage);

        let end_timestamp = env.block.time.plus_seconds(100);
        // Create one proposal
        let proposal_id = SIMPLE_VOTING
            .new_proposal(
                storage,
                end_timestamp,
                &[Addr::unchecked("alice"), Addr::unchecked("bob")],
            )
            .unwrap();
        assert_eq!(proposal_id, 1);

        let proposal = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 0,
                votes_against: 0,
                status: ProposalStatus::Active,
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                end_timestamp
            }
        );

        // Create another proposal (already expired)
        let proposal_id = SIMPLE_VOTING
            .new_proposal(
                storage,
                env.block.time,
                &[Addr::unchecked("alice"), Addr::unchecked("bob")],
            )
            .unwrap();
        assert_eq!(proposal_id, 2);

        let proposal = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 0,
                votes_against: 0,
                status: ProposalStatus::WaitingForCount,
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                end_timestamp: env.block.time
            }
        );
    }

    #[test]
    fn create_proposal_duplicate_friends() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let storage = &mut deps.storage;
        default_setup(storage);

        let end_timestamp = env.block.time.plus_seconds(100);

        let err = SIMPLE_VOTING
            .new_proposal(
                storage,
                end_timestamp,
                &[Addr::unchecked("alice"), Addr::unchecked("alice")],
            )
            .unwrap_err();
        assert_eq!(err, VoteError::DuplicateAddrs {});
    }

    #[test]
    fn cancel_vote() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let storage = &mut deps.storage;

        default_setup(storage);

        let end_timestamp = env.block.time.plus_seconds(100);
        // Create one proposal
        let proposal_id = SIMPLE_VOTING
            .new_proposal(
                storage,
                end_timestamp,
                &[Addr::unchecked("alice"), Addr::unchecked("bob")],
            )
            .unwrap();

        SIMPLE_VOTING
            .cancel_proposal(storage, &env.block, proposal_id)
            .unwrap();

        let proposal = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 0,
                votes_against: 0,
                status: ProposalStatus::Finished(ProposalOutcome::Canceled),
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                // Finish time here
                end_timestamp: env.block.time
            }
        );

        // Can't cancel during non-active
        let err = SIMPLE_VOTING
            .cancel_proposal(storage, &env.block, proposal_id)
            .unwrap_err();
        assert_eq!(
            err,
            VoteError::ProposalNotActive(ProposalStatus::Finished(ProposalOutcome::Canceled))
        );
    }

    // Check it updates status when required
    #[test]
    fn load_proposal() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let storage = &mut deps.storage;
        setup(
            storage,
            &VoteConfig {
                threshold: Threshold::Majority {},
                veto_duration_seconds: Some(Uint64::new(10)),
            },
        );

        let end_timestamp = env.block.time.plus_seconds(100);
        let proposal_id = SIMPLE_VOTING
            .new_proposal(storage, end_timestamp, &[Addr::unchecked("alice")])
            .unwrap();
        let proposal: ProposalInfo = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(proposal.status, ProposalStatus::Active,);

        // Should auto-update to the veto
        env.block.time = end_timestamp;
        let proposal: ProposalInfo = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::VetoPeriod(end_timestamp.plus_seconds(10)),
        );

        // Should update to the WaitingForCount
        env.block.time = end_timestamp.plus_seconds(10);
        let proposal: ProposalInfo = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(proposal.status, ProposalStatus::WaitingForCount,);

        // Should update to the Finished
        SIMPLE_VOTING
            .count_votes(storage, &env.block, proposal_id)
            .unwrap();
        let proposal: ProposalInfo = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert!(matches!(proposal.status, ProposalStatus::Finished(_)));

        SIMPLE_VOTING
            .update_vote_config(
                storage,
                &VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None,
                },
            )
            .unwrap();

        let end_timestamp = env.block.time.plus_seconds(100);
        let proposal_id = SIMPLE_VOTING
            .new_proposal(storage, end_timestamp, &[Addr::unchecked("alice")])
            .unwrap();
        // Should auto-update to the waiting if not configured veto period
        env.block.time = end_timestamp;
        let proposal: ProposalInfo = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(proposal.status, ProposalStatus::WaitingForCount,);
    }

    #[test]
    fn cast_vote() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let storage = &mut deps.storage;
        default_setup(storage);

        let end_timestamp = env.block.time.plus_seconds(100);
        let proposal_id = SIMPLE_VOTING
            .new_proposal(
                storage,
                end_timestamp,
                &[Addr::unchecked("alice"), Addr::unchecked("bob")],
            )
            .unwrap();

        // Alice vote
        SIMPLE_VOTING
            .cast_vote(
                deps.as_mut().storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("alice"),
                Vote {
                    vote: false,
                    memo: None,
                },
            )
            .unwrap();
        let vote = SIMPLE_VOTING
            .load_vote(
                deps.as_ref().storage,
                proposal_id,
                &Addr::unchecked("alice"),
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            vote,
            Vote {
                vote: false,
                memo: None
            }
        );
        let proposal = SIMPLE_VOTING
            .load_proposal(deps.as_ref().storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 0,
                votes_against: 1,
                status: ProposalStatus::Active,
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                end_timestamp
            }
        );

        // Bob votes
        SIMPLE_VOTING
            .cast_vote(
                deps.as_mut().storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("bob"),
                Vote {
                    vote: false,
                    memo: Some("memo".to_owned()),
                },
            )
            .unwrap();
        let vote = SIMPLE_VOTING
            .load_vote(deps.as_ref().storage, proposal_id, &Addr::unchecked("bob"))
            .unwrap()
            .unwrap();
        assert_eq!(
            vote,
            Vote {
                vote: false,
                memo: Some("memo".to_owned())
            }
        );
        let proposal = SIMPLE_VOTING
            .load_proposal(deps.as_ref().storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 0,
                votes_against: 2,
                status: ProposalStatus::Active,
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                end_timestamp
            }
        );

        // re-cast votes(to the same vote)
        SIMPLE_VOTING
            .cast_vote(
                deps.as_mut().storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("alice"),
                Vote {
                    vote: false,
                    memo: None,
                },
            )
            .unwrap();
        // unchanged
        let proposal = SIMPLE_VOTING
            .load_proposal(deps.as_ref().storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 0,
                votes_against: 2,
                status: ProposalStatus::Active,
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                end_timestamp
            }
        );

        // re-cast votes(to the opposite vote)
        SIMPLE_VOTING
            .cast_vote(
                deps.as_mut().storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("bob"),
                Vote {
                    vote: true,
                    memo: None,
                },
            )
            .unwrap();
        // unchanged
        let proposal = SIMPLE_VOTING
            .load_proposal(deps.as_ref().storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 1,
                votes_against: 1,
                status: ProposalStatus::Active,
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                end_timestamp
            }
        );
    }

    #[test]
    fn invalid_cast_votes() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let storage = &mut deps.storage;
        setup(
            storage,
            &VoteConfig {
                threshold: Threshold::Majority {},
                veto_duration_seconds: Some(Uint64::new(10)),
            },
        );

        let end_timestamp = env.block.time.plus_seconds(100);
        let proposal_id = SIMPLE_VOTING
            .new_proposal(
                storage,
                end_timestamp,
                &[Addr::unchecked("alice"), Addr::unchecked("bob")],
            )
            .unwrap();

        // Stranger vote
        let err = SIMPLE_VOTING
            .cast_vote(
                deps.as_mut().storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("stranger"),
                Vote {
                    vote: false,
                    memo: None,
                },
            )
            .unwrap_err();
        assert_eq!(err, VoteError::Unauthorized {});

        // Vote during veto
        env.block.time = end_timestamp;

        // Vote during veto
        let err = SIMPLE_VOTING
            .cast_vote(
                deps.as_mut().storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("alice"),
                Vote {
                    vote: false,
                    memo: None,
                },
            )
            .unwrap_err();
        assert_eq!(
            err,
            VoteError::ProposalNotActive(ProposalStatus::VetoPeriod(
                env.block.time.plus_seconds(10)
            ))
        );

        env.block.time = env.block.time.plus_seconds(10);

        // Too late vote
        let err = SIMPLE_VOTING
            .cast_vote(
                deps.as_mut().storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("alice"),
                Vote {
                    vote: false,
                    memo: None,
                },
            )
            .unwrap_err();
        assert_eq!(
            err,
            VoteError::ProposalNotActive(ProposalStatus::WaitingForCount)
        );

        // Post-finish votes
        SIMPLE_VOTING
            .count_votes(deps.as_mut().storage, &env.block, proposal_id)
            .unwrap();
        let err = SIMPLE_VOTING
            .cast_vote(
                deps.as_mut().storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("alice"),
                Vote {
                    vote: false,
                    memo: None,
                },
            )
            .unwrap_err();
        assert_eq!(
            err,
            VoteError::ProposalNotActive(ProposalStatus::Finished(ProposalOutcome::Failed))
        );
    }

    #[test]
    fn count_votes() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let storage = &mut deps.storage;
        default_setup(storage);

        // Failed proposal
        let end_timestamp = env.block.time.plus_seconds(100);
        let proposal_id = SIMPLE_VOTING
            .new_proposal(
                storage,
                end_timestamp,
                &[Addr::unchecked("alice"), Addr::unchecked("bob")],
            )
            .unwrap();
        env.block.time = end_timestamp.plus_seconds(10);
        SIMPLE_VOTING
            .count_votes(storage, &env.block, proposal_id)
            .unwrap();
        let proposal = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 0,
                votes_against: 0,
                status: ProposalStatus::Finished(ProposalOutcome::Failed),
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                end_timestamp: end_timestamp.plus_seconds(10)
            }
        );

        // Succeeded proposal 2/3 majority
        let end_timestamp = env.block.time.plus_seconds(100);
        let proposal_id = SIMPLE_VOTING
            .new_proposal(
                storage,
                end_timestamp,
                &[
                    Addr::unchecked("alice"),
                    Addr::unchecked("bob"),
                    Addr::unchecked("afk"),
                ],
            )
            .unwrap();
        SIMPLE_VOTING
            .cast_vote(
                storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("alice"),
                Vote {
                    vote: true,
                    memo: None,
                },
            )
            .unwrap();
        SIMPLE_VOTING
            .cast_vote(
                storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("bob"),
                Vote {
                    vote: true,
                    memo: None,
                },
            )
            .unwrap();
        env.block.time = end_timestamp;
        SIMPLE_VOTING
            .count_votes(storage, &env.block, proposal_id)
            .unwrap();
        let proposal = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 3,
                votes_for: 2,
                votes_against: 0,
                status: ProposalStatus::Finished(ProposalOutcome::Passed),
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None
                },
                end_timestamp
            }
        );

        // Succeeded proposal 1/2 50% Decimal
        SIMPLE_VOTING
            .update_vote_config(
                storage,
                &VoteConfig {
                    threshold: Threshold::Percentage(Decimal::percent(50)),
                    veto_duration_seconds: None,
                },
            )
            .unwrap();
        let end_timestamp = env.block.time.plus_seconds(100);
        let proposal_id = SIMPLE_VOTING
            .new_proposal(
                storage,
                end_timestamp,
                &[Addr::unchecked("alice"), Addr::unchecked("bob")],
            )
            .unwrap();
        SIMPLE_VOTING
            .cast_vote(
                storage,
                &env.block,
                proposal_id,
                &Addr::unchecked("alice"),
                Vote {
                    vote: true,
                    memo: None,
                },
            )
            .unwrap();

        env.block.time = end_timestamp;
        SIMPLE_VOTING
            .count_votes(storage, &env.block, proposal_id)
            .unwrap();
        let proposal = SIMPLE_VOTING
            .load_proposal(storage, &env.block, proposal_id)
            .unwrap();
        assert_eq!(
            proposal,
            ProposalInfo {
                total_voters: 2,
                votes_for: 1,
                votes_against: 0,
                status: ProposalStatus::Finished(ProposalOutcome::Passed),
                config: VoteConfig {
                    threshold: Threshold::Percentage(Decimal::percent(50)),
                    veto_duration_seconds: None
                },
                end_timestamp
            }
        );
    }
}
