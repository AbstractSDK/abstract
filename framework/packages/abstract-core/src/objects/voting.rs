use crate::AbstractResult;
use cosmwasm_std::{ensure, Addr, BlockInfo, Decimal, Env, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use cw_utils::{Duration, Expiration};

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

    pub fn instantiate(
        &self,
        store: &mut dyn Storage,
        vote_config: &VoteConfig,
    ) -> AbstractResult<()> {
        self.next_vote_id.save(store, &VoteId::default())?;
        self.vote_config.save(store, vote_config)?;
        Ok(())
    }

    pub fn new_vote(
        &self,
        store: &mut dyn Storage,
        initial_voters: &[Addr],
    ) -> AbstractResult<VoteId> {
        let vote_id = self
            .next_vote_id
            .update(store, |id| AbstractResult::Ok(id + 1))?;

        self.vote_info.save(
            store,
            vote_id,
            &VoteInfo {
                total_voters: initial_voters.len() as u32,
                status: VoteStatus::Active {},
            },
        )?;
        for voter in initial_voters {
            self.votes.save(store, (vote_id, voter), &None)?;
        }
        Ok(vote_id)
    }

    pub fn cast_vote(
        &self,
        store: &mut dyn Storage,
        block_info: &BlockInfo,
        vote_id: VoteId,
        voter: &Addr,
        vote: Vote,
        // new_voter: bool,
    ) -> AbstractResult<VoteInfo> {
        // Need to check it's existing vote
        let mut vote_info =
            self.vote_info
                .may_load(store, vote_id)?
                .ok_or(crate::AbstractError::Std(StdError::generic_err(
                    "There are no vote by this id",
                )))?;
        match &vote_info.status {
            // If veto configured need to update vote info
            VoteStatus::Active {} => {
                let vote_config = self.vote_config.load(store)?;
                if let Some(duration) = vote_config.veto_duration {
                    vote_info.status = VoteStatus::VetoPeriod(duration.after(block_info))
                }
                self.vote_info.save(store, vote_id, &vote_info)?;
            }
            VoteStatus::VetoPeriod(_) => {}
            VoteStatus::Finished(_) => {
                return Err(crate::AbstractError::Std(StdError::generic_err(
                    "This vote is over",
                )));
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
                Some(_v) => Ok(Some(vote)),
                None => Err(StdError::generic_err("This user is not allowed to vote")),
            },
        )?;
        // }
        // }
        Ok(vote_info)
    }

    pub fn count_votes(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
    ) -> AbstractResult<VoteInfo> {        
        let mut vote_info =
            self.vote_info
                .may_load(store, vote_id)?
                .ok_or(crate::AbstractError::Std(StdError::generic_err(
                    "There are no vote by this id",
                )))?;
        if let VoteStatus::Finished(_) = &vote_info.status {
            return Err(crate::AbstractError::Std(StdError::generic_err(
                "This vote is over",
            )));
        }
        let vote_config = self.vote_config.load(store)?;

        // Calculate votes
        let votes = self.votes_for_id(store, vote_id)?;
        let positive_votes = votes.iter().fold(0u128, |acc, (_voter, vote)| {
            if matches!(vote, Some(Vote { vote: true, .. })) {
                acc + 1
            } else {
                acc
            }
        });
        let threshold = match vote_config.threshold {
            Threshold::Majority {} => Uint128::from(vote_info.total_voters / 2),
            Threshold::Percentage(decimal) => decimal * Uint128::from(vote_info.total_voters),
        };
        
        // Update vote status
        vote_info.status = if Uint128::new(positive_votes) > threshold {
            VoteStatus::Finished(VoteResult::Passed {})
        } else {
            VoteStatus::Finished(VoteResult::Failed {})
        };
        self.vote_info.save(store, vote_id, &vote_info)?;

        Ok(vote_info)
    }

    pub fn add_voters(
        &self,
        store: &mut dyn Storage,
        vote_id: VoteId,
        new_voters: &[Addr],
    ) -> AbstractResult<()> {
        // Need to check it's existing vote
        let mut vote_info =
            self.vote_info
                .may_load(store, vote_id)?
                .ok_or(crate::AbstractError::Std(StdError::generic_err(
                    "There are no vote by this id",
                )))?;
        if let VoteStatus::Finished(_) = &vote_info.status {
            return Err(crate::AbstractError::Std(StdError::generic_err(
                "This vote is over",
            )));
        }

        for voter in new_voters {
            // Don't override already existing vote
            self.votes.update(store, (vote_id, voter), |v| match v {
                Some(v) => AbstractResult::Ok(v),
                None => {
                    vote_info.total_voters += 1;
                    AbstractResult::Ok(None)
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
        removed_voters: &[Addr],
    ) -> AbstractResult<()> {
        let mut vote_info =
            self.vote_info
                .may_load(store, vote_id)?
                .ok_or(crate::AbstractError::Std(StdError::generic_err(
                    "There are no vote by this id",
                )))?;

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
    ) -> AbstractResult<Option<Vote>> {
        self.votes.load(store, (vote_id, voter)).map_err(Into::into)
    }

    // TODO: do we want pagination on this?
    // I guess no, because if we can't load all votes in one tx means we can't count it
    // so maybe another TODO: Figure out max len for it
    pub fn votes_for_id(
        &self,
        store: &dyn Storage,
        vote_id: VoteId,
    ) -> AbstractResult<Vec<(Addr, Option<Vote>)>> {
        let votes = self
            .votes
            .prefix(vote_id)
            .range(store, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<_>>()?;
        Ok(votes)
    }

    pub fn query_list(
        &self,
        store: &dyn Storage,
        start_after: Option<(VoteId, &Addr)>,
        limit: Option<u64>,
    ) -> AbstractResult<Vec<((VoteId, Addr), Option<Vote>)>> {
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
    pub status: VoteStatus,
}

#[cosmwasm_schema::cw_serde]
pub enum VoteStatus {
    Active {},
    VetoPeriod(Expiration),
    Finished(VoteResult),
}

#[cosmwasm_schema::cw_serde]
pub enum VoteResult {
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
    fn validate_percentage(&self) -> StdResult<()> {
        if let Threshold::Percentage(percent) = self {
            if percent.is_zero() {
                Err(StdError::generic_err("Threshold can't be 0%"))
            } else if *percent > Decimal::one() {
                Err(StdError::generic_err(
                    "Not possible to reach >100% threshold",
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
