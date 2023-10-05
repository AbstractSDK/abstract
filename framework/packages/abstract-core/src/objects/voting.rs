use cosmwasm_std::{Addr, BlockInfo, Decimal, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use cw_utils::{Duration, Expiration};
use thiserror::Error;

/// Wrapper error for the Abstract framework.
#[derive(Error, Debug, PartialEq)]
pub enum VoteError {
    #[error("Std error encountered while handling voting object: {0}")]
    Std(#[from] StdError),

    #[error("No vote by vote id: {vote_id}")]
    NoVoteById { vote_id: VoteId },

    #[error("No actions allowed on finished voting, vote id: {vote_id}")]
    AlreadyFinished { vote_id: VoteId },

    #[error("Threshold error: {0}")]
    ThresholdError(String),

    #[error("Veto period expired, can't cancel vote: {vote_id}, {expiration}")]
    VetoExpired {
        vote_id: VoteId,
        expiration: Expiration,
    },

    #[error("Vote period expired, can't do actions: {vote_id}, {expiration}")]
    VoteExpired {
        vote_id: VoteId,
        expiration: Expiration,
    },
}

pub type VoteResult<T> = Result<T, VoteError>;

// TODO: custom error type

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
        vote_info.assert_ready_for_action(vote_id, block)?;

        // If veto configured need to update vote info
        if let VoteStatus::Active {} = &vote_info.status {
            let vote_config = self.vote_config.load(store)?;
            if let Some(duration) = vote_config.veto_duration {
                let veto_exp = duration.after(block);
                let expiration = if veto_exp < vote_info.end {
                    veto_exp
                } else {
                    vote_info.end
                };
                vote_info.status = VoteStatus::VetoPeriod(expiration)

            }
        }
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
        vote_info.assert_ready_for_action(vote_id, block)?;

        if let VoteStatus::VetoPeriod(expiration) = vote_info.status {
            if expiration.is_expired(block) {
                return Err(VoteError::VetoExpired {
                    vote_id,
                    expiration,
                });
            }
        }
        vote_info.status = VoteStatus::Finished(VoteOutcome::Canceled {});
        self.vote_info.save(store, vote_id, &vote_info)?;
        Ok(vote_info)
    }

    pub fn count_votes(&self, store: &mut dyn Storage, vote_id: VoteId) -> VoteResult<VoteInfo> {
        let mut vote_info = self.load_vote_info(store, vote_id)?;
        vote_info.status.assert_not_finished(vote_id)?;

        let vote_config = self.vote_config.load(store)?;

        // Calculate votes
        let votes_for = vote_info.votes_for;
        let threshold = match vote_config.threshold {
            Threshold::Majority {} => Uint128::from(vote_info.total_voters / 2),
            Threshold::Percentage(decimal) => decimal * Uint128::from(vote_info.total_voters),
        };

        // Update vote status
        vote_info.status = if Uint128::from(votes_for) > threshold {
            VoteStatus::Finished(VoteOutcome::Passed {})
        } else {
            VoteStatus::Finished(VoteOutcome::Failed {})
        };
        self.vote_info.save(store, vote_id, &vote_info)?;

        Ok(vote_info)
    }

    pub fn add_voters(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
        block: &BlockInfo,
        new_voters: &[Addr],
    ) -> VoteResult<()> {
        // Need to check it's existing vote
        let mut vote_info = self.load_vote_info(store, vote_id)?;
        vote_info.assert_ready_for_action(vote_id, block)?;

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

        Ok(())
    }

    pub fn remove_voters(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
        block: &BlockInfo,
        removed_voters: &[Addr],
    ) -> VoteResult<()> {
        let mut vote_info = self.load_vote_info(store, vote_id)?;
        vote_info.assert_ready_for_action(vote_id, block)?;

        for voter in removed_voters {
            // Would be nice to get this fixed:
            // https://github.com/CosmWasm/cosmwasm/issues/290
            if self.votes.has(store, (vote_id, voter)) {
                vote_info.total_voters -= 1;
                self.votes.remove(store, (vote_id, voter));
            }
        }
        self.vote_info.save(store, vote_id, &vote_info)?;
        Ok(())
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
            .ok_or(VoteError::NoVoteById { vote_id })
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

#[cosmwasm_schema::cw_serde]
pub struct Vote {
    pub vote: bool,
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

    pub fn assert_ready_for_action(&self, vote_id: VoteId, block: &BlockInfo) -> VoteResult<()> {
        self.status.assert_not_finished(vote_id)?;
        if self.end.is_expired(block) {
            Err(VoteError::VoteExpired {
                vote_id,
                expiration: self.end,
            })
        } else {
            Ok(())
        }
    }
}

impl VoteInfo {
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
    Active {},
    VetoPeriod(Expiration),
    Finished(VoteOutcome),
}

impl VoteStatus {
    pub fn assert_not_finished(&self, vote_id: VoteId) -> VoteResult<()> {
        match self {
            VoteStatus::Finished(_) => Err(VoteError::AlreadyFinished { vote_id }),
            _ => Ok(()),
        }
    }
}

// TODO: do we want more info like BlockInfo?
#[cosmwasm_schema::cw_serde]
pub enum VoteOutcome {
    Passed {},
    Failed {},
    Canceled {},
}

#[cosmwasm_schema::cw_serde]
pub struct VoteConfig {
    pub threshold: Threshold,
    // Veto duration after the first vote
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
